using System.Net.Sockets;
using System.Security.Cryptography;
using System.Text;
using System.Text.Json;
using System.Text.Json.Nodes;
using System.Text.Json.Serialization;

namespace RstrBot.Plugin;

public sealed record InstanceContext(
    [property: JsonPropertyName("workspace_id")] string WorkspaceId,
    [property: JsonPropertyName("instance_id")] string InstanceId,
    [property: JsonPropertyName("generation")] long Generation);

public sealed class RpcFailure(int code, string message) : Exception(message)
{
    public int Code { get; } = code;
}

public sealed class CallContext
{
    private readonly Func<JsonObject, CancellationToken, Task> send;
    internal CallContext(string requestId, InstanceContext instance,
        Func<JsonObject, CancellationToken, Task> sendFrame)
    {
        RequestId = requestId;
        Instance = instance;
        send = sendFrame;
    }
    public string RequestId { get; }
    public InstanceContext Instance { get; }
    public Task EmitAsync(string method, JsonObject parameters, CancellationToken cancellation)
    {
        if (method is not ("model.delta" or "tool.progress" or "platform.progress" or "task.progress"))
            throw new RpcFailure(-32602, "Unsupported event method");
        var body = (JsonObject)parameters.DeepClone();
        body["request_id"] = RequestId;
        return send(new JsonObject { ["jsonrpc"] = "2.0", ["method"] = method, ["params"] = body }, cancellation);
    }
}

public static class PluginServer
{
    public const string ProtocolVersion = "1.0";
    public const int MaxFrameBytes = 1024 * 1024;

    public static async Task ServeAsync(string[] capabilities,
        Func<string, JsonObject, CallContext, CancellationToken, Task<JsonNode?>> handler,
        CancellationToken stopping = default)
    {
        string Required(string key) => Environment.GetEnvironmentVariable(key)
            ?? throw new InvalidOperationException("Host launch environment required");
        var path = Required("RSTRBOT_SOCKET");
        var nonce = Encoding.UTF8.GetBytes(Required("RSTRBOT_TOKEN"));
        var context = JsonSerializer.Deserialize<InstanceContext>(Required("RSTRBOT_CONTEXT"))
            ?? throw new InvalidOperationException("Invalid instance context");
        using var socket = new Socket(AddressFamily.Unix, SocketType.Stream, ProtocolType.Unspecified);
        await socket.ConnectAsync(new UnixDomainSocketEndPoint(path), stopping);
        await using var stream = new NetworkStream(socket, ownsSocket: false);
        using var writeLock = new SemaphoreSlim(1);
        var reader = new FrameReader(stream);
        async Task Send(JsonObject value, CancellationToken cancellation)
        {
            var bytes = JsonSerializer.SerializeToUtf8Bytes(value);
            if (bytes.Length > MaxFrameBytes) throw new RpcFailure(-32602, "Frame exceeds byte limit");
            await writeLock.WaitAsync(cancellation);
            try
            {
                await stream.WriteAsync(bytes, cancellation);
                await stream.WriteAsync("\n"u8.ToArray(), cancellation);
                await stream.FlushAsync(cancellation);
            }
            finally { writeLock.Release(); }
        }
        var hello = await reader.ReadAsync(stopping);
        var parameters = hello["params"] as JsonObject ?? new JsonObject();
        var receivedNonce = Encoding.UTF8.GetBytes(parameters["token"]?.GetValue<string>() ?? "");
        var remote = parameters["context"]?.Deserialize<InstanceContext>();
        if (hello["method"]?.GetValue<string>() != "initialize"
            || parameters["protocol_version"]?.GetValue<string>() != ProtocolVersion
            || !CryptographicOperations.FixedTimeEquals(nonce, receivedNonce) || remote != context)
        {
            await Send(Error(hello["id"]?.DeepClone(), -32001, "Initialization rejected"), stopping);
            return;
        }
        var initialized = JsonSerializer.SerializeToNode(context)!.AsObject();
        initialized["protocol_version"] = ProtocolVersion;
        initialized["capabilities"] = JsonSerializer.SerializeToNode(capabilities);
        await Send(new JsonObject { ["jsonrpc"] = "2.0", ["id"] = hello["id"]?.DeepClone(), ["result"] = initialized }, stopping);
        var request = await reader.ReadAsync(stopping);
        var id = request["id"]?.GetValue<string>() ?? throw new RpcFailure(-32600, "Request id required");
        var method = request["method"]?.GetValue<string>() ?? throw new RpcFailure(-32600, "Method required");
        using var cancellation = CancellationTokenSource.CreateLinkedTokenSource(stopping);
        var call = new CallContext(id, context, Send);
        var operation = handler(method, request["params"] as JsonObject ?? new JsonObject(), call, cancellation.Token);
        var monitor = reader.ReadAsync(cancellation.Token);
        try
        {
            var first = await Task.WhenAny(operation, monitor);
            if (first != operation)
            {
                await cancellation.CancelAsync();
                throw new RpcFailure(-32800, "Invocation cancelled or host disconnected");
            }
            var result = await operation;
            await Send(new JsonObject { ["jsonrpc"] = "2.0", ["id"] = id, ["result"] = result }, stopping);
        }
        catch (RpcFailure error)
        {
            await Send(Error(JsonValue.Create(id), error.Code, error.Message[..Math.Min(error.Message.Length, 500)]), stopping);
        }
        catch (OperationCanceledException)
        {
            await Send(Error(JsonValue.Create(id), -32800, "Invocation cancelled"), stopping);
        }
        catch (Exception)
        {
            await Send(Error(JsonValue.Create(id), -32603, "Plugin handler failed"), stopping);
        }
        finally
        {
            await cancellation.CancelAsync();
            try { await monitor; } catch (Exception) { /* Completion/EOF is expected during cleanup. */ }
        }
    }

    private static JsonObject Error(JsonNode? id, int code, string message) => new()
    {
        ["jsonrpc"] = "2.0", ["id"] = id,
        ["error"] = new JsonObject { ["code"] = code, ["message"] = message }
    };

    private sealed class FrameReader(Stream stream)
    {
        private readonly byte[] buffer = new byte[8192];
        private int offset;
        private int count;
        public async Task<JsonObject> ReadAsync(CancellationToken cancellation)
        {
            using var frame = new MemoryStream();
            while (true)
            {
                if (offset == count)
                {
                    count = await stream.ReadAsync(buffer, cancellation);
                    offset = 0;
                    if (count == 0) throw new IOException("Host disconnected");
                }
                var next = Array.IndexOf(buffer, (byte)'\n', offset, count - offset);
                var length = (next < 0 ? count : next) - offset;
                if (frame.Length + length > MaxFrameBytes) throw new RpcFailure(-32600, "Frame exceeds byte limit");
                frame.Write(buffer, offset, length);
                offset += length;
                if (next < 0) continue;
                offset++;
                var value = JsonNode.Parse(frame.ToArray()) as JsonObject
                    ?? throw new RpcFailure(-32600, "JSON-RPC object required");
                if (value["jsonrpc"]?.GetValue<string>() != "2.0") throw new RpcFailure(-32600, "JSON-RPC 2.0 required");
                return value;
            }
        }
    }
}
