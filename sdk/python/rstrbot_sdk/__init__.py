"""RstrBot protocol 1.0; no dependency on the core's private objects."""
from __future__ import annotations

import argparse
import asyncio
from dataclasses import dataclass
import hmac
from importlib import metadata
import json
import os
from typing import Any, Awaitable, Callable

PROTOCOL_VERSION = "1.0"
MAX_FRAME_BYTES = 1024 * 1024
EVENTS = frozenset(("model.delta", "tool.progress", "platform.progress", "task.progress"))


@dataclass
class RpcFailure(Exception):
    code: int
    message: str


@dataclass(frozen=True)
class Call:
    request_id: str
    context: dict[str, Any]
    cancelled: asyncio.Event
    _send: Callable[[dict[str, Any]], Awaitable[None]]

    async def emit(self, method: str, params: dict[str, Any]) -> None:
        if method not in EVENTS:
            raise RpcFailure(-32602, "Unsupported event method")
        if self.cancelled.is_set():
            raise RpcFailure(-32800, "Invocation cancelled")
        await self._send({"jsonrpc": "2.0", "method": method,
                          "params": {**params, "request_id": self.request_id}})


Handler = Callable[[str, dict[str, Any], Call], Awaitable[Any]]


@dataclass(frozen=True)
class Plugin:
    capabilities: tuple[str, ...]
    handle: Handler


def discover() -> list[dict[str, str]]:
    """Read entry-point metadata only; scanning never executes plugin modules."""
    return [{"name": item.name, "value": item.value,
             "distribution": item.dist.name if item.dist else ""}
            for item in metadata.entry_points(group="rstrbot.plugins")]


async def serve(plugin: Plugin) -> None:
    """Serve one authenticated host invocation over a private Unix socket."""
    reader, writer = await asyncio.open_unix_connection(
        os.environ["RSTRBOT_SOCKET"], limit=MAX_FRAME_BYTES + 1)
    lock = asyncio.Lock()

    async def send(frame: dict[str, Any]) -> None:
        encoded = json.dumps(frame, ensure_ascii=False, separators=(",", ":")).encode()
        if len(encoded) > MAX_FRAME_BYTES:
            raise RpcFailure(-32602, "Frame exceeds byte limit")
        async with lock:
            writer.write(encoded + b"\n")
            await writer.drain()

    async def receive() -> dict[str, Any]:
        line = await reader.readline()
        if not line or len(line) > MAX_FRAME_BYTES + 1 or not line.endswith(b"\n"):
            raise RpcFailure(-32600, "Invalid or oversized frame")
        frame = json.loads(line)
        if not isinstance(frame, dict) or frame.get("jsonrpc") != "2.0":
            raise RpcFailure(-32600, "JSON-RPC 2.0 object required")
        return frame

    try:
        hello = await receive()
        params = hello.get("params", {})
        expected = json.loads(os.environ["RSTRBOT_CONTEXT"])
        token = params.get("token", "")
        valid = (hello.get("method") == "initialize" and isinstance(hello.get("id"), str)
                 and params.get("protocol_version") == PROTOCOL_VERSION
                 and isinstance(token, str)
                 and hmac.compare_digest(token, os.environ["RSTRBOT_TOKEN"])
                 and params.get("context") == expected)
        if not valid:
            await send({"jsonrpc": "2.0", "id": hello.get("id"),
                        "error": {"code": -32001, "message": "Initialization rejected"}})
            return
        await send({"jsonrpc": "2.0", "id": hello["id"], "result": {
            "protocol_version": PROTOCOL_VERSION, "capabilities": plugin.capabilities, **expected}})
        request = await receive()
        request_id, method = request.get("id"), request.get("method")
        if not isinstance(request_id, str) or not isinstance(method, str):
            raise RpcFailure(-32600, "Request id and method required")
        cancelled = asyncio.Event()
        call = Call(request_id, expected.copy(), cancelled, send)
        invocation = asyncio.create_task(plugin.handle(method, request.get("params", {}), call))
        monitor = asyncio.create_task(receive())
        try:
            done, _ = await asyncio.wait((invocation, monitor), return_when=asyncio.FIRST_COMPLETED)
            if invocation in done:
                result = await invocation
            else:
                cancelled.set()
                invocation.cancel()
                raise RpcFailure(-32800, "Invocation cancelled or host disconnected")
            await send({"jsonrpc": "2.0", "id": request_id, "result": result})
        except RpcFailure as error:
            await send({"jsonrpc": "2.0", "id": request_id, "error": {
                "code": error.code, "message": error.message[:500]}})
        except Exception:
            # Do not leak secrets contained in exception arguments or URLs.
            await send({"jsonrpc": "2.0", "id": request_id, "error": {
                "code": -32603, "message": "Plugin handler failed"}})
        finally:
            monitor.cancel()
            invocation.cancel()
            await asyncio.gather(monitor, invocation, return_exceptions=True)
    finally:
        writer.close()
        await writer.wait_closed()


def main() -> None:
    parser = argparse.ArgumentParser(description="Load a registered RstrBot Python entry point")
    parser.add_argument("--list", action="store_true", help="Print metadata without importing plugins")
    parser.add_argument("--entry-point")
    options = parser.parse_args()
    if options.list:
        print(json.dumps(discover(), ensure_ascii=False))
        return
    matches = [item for item in metadata.entry_points(group="rstrbot.plugins")
               if item.name == options.entry_point]
    if len(matches) != 1:
        parser.error("Select exactly one installed entry point; ambiguous or missing names are rejected")
    plugin = matches[0].load()
    if not isinstance(plugin, Plugin):
        parser.error("Entry point must export rstrbot_sdk.Plugin")
    asyncio.run(serve(plugin))
