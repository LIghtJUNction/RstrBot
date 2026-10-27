import { createConnection, type Socket } from 'node:net';
import { timingSafeEqual } from 'node:crypto';
import { once } from 'node:events';

export const PROTOCOL_VERSION = '1.0';
export const MAX_FRAME_BYTES = 1024 * 1024;
export type Context = { workspace_id: string; instance_id: string; generation: number };
export type Frame = Record<string, unknown>;
export class RpcFailure extends Error {
  constructor(public readonly code: number, message: string) { super(message); }
}
export type Call = {
  request_id: string; context: Readonly<Context>; signal: AbortSignal;
  emit(method: string, params: Frame): Promise<void>;
};
export type Handler = (method: string, params: Frame, call: Call) => Promise<unknown>;
const eventMethods = new Set(['model.delta', 'tool.progress', 'platform.progress', 'task.progress']);

function frameReader(socket: Socket) {
  let buffer = Buffer.alloc(0);
  const frames: Frame[] = [];
  let pending: { resolve(value: Frame): void; reject(error: Error): void } | undefined;
  let failure: Error | undefined;
  const fail = (error: Error) => { failure = error; pending?.reject(error); pending = undefined; };
  socket.on('error', fail);
  socket.on('close', () => fail(new Error('Host disconnected')));
  socket.on('data', (data: Buffer) => {
    // Bound both the incomplete frame and the number of queued frames.
    if (buffer.length + data.length > MAX_FRAME_BYTES + 1) {
      fail(new Error('Frame exceeds byte limit')); socket.destroy(); return;
    }
    buffer = Buffer.concat([buffer, data]);
    for (;;) {
      const end = buffer.indexOf(10);
      if (end < 0) break;
      const line = buffer.subarray(0, end); buffer = buffer.subarray(end + 1);
      try {
        const value: unknown = JSON.parse(line.toString('utf8'));
        if (!value || typeof value !== 'object' || Array.isArray(value) || (value as Frame).jsonrpc !== '2.0')
          throw new Error('JSON-RPC 2.0 object required');
        if (pending) { const next = pending; pending = undefined; next.resolve(value as Frame); }
        else if (frames.length < 4) frames.push(value as Frame);
        else throw new Error('Too many queued frames');
      } catch { fail(new Error('Invalid host frame')); socket.destroy(); return; }
    }
  });
  return () => new Promise<Frame>((resolve, reject) => {
    const frame = frames.shift();
    if (frame) resolve(frame);
    else if (failure) reject(failure);
    else if (pending) reject(new Error('Only one reader is allowed'));
    else pending = { resolve, reject };
  });
}

/** Serve one authenticated local invocation; no raw core objects are exposed. */
export async function serve(capabilities: readonly string[], handler: Handler): Promise<void> {
  const socketPath = process.env.RSTRBOT_SOCKET;
  const secret = process.env.RSTRBOT_TOKEN;
  if (!socketPath || !secret || !process.env.RSTRBOT_CONTEXT) throw new Error('Host launch environment required');
  const context = JSON.parse(process.env.RSTRBOT_CONTEXT) as Context;
  const socket = createConnection(socketPath);
  const receive = frameReader(socket);
  const send = async (frame: Frame) => {
    const bytes = Buffer.from(JSON.stringify(frame) + '\n');
    if (bytes.length > MAX_FRAME_BYTES + 1) throw new RpcFailure(-32602, 'Frame exceeds byte limit');
    if (!socket.write(bytes)) await once(socket, 'drain');
  };
  try {
    await once(socket, 'connect');
    const hello = await receive();
    const params = (hello.params ?? {}) as Frame;
    const nonce = Buffer.from(typeof params.token === 'string' ? params.token : '');
    const expected = Buffer.from(secret);
    const remoteContext = params.context as Context | undefined;
    if (hello.method !== 'initialize' || typeof hello.id !== 'string'
        || params.protocol_version !== PROTOCOL_VERSION || nonce.length !== expected.length
        || !timingSafeEqual(nonce, expected) || remoteContext?.workspace_id !== context.workspace_id
        || remoteContext?.instance_id !== context.instance_id || remoteContext?.generation !== context.generation) {
      await send({ jsonrpc: '2.0', id: hello.id ?? null, error: { code: -32001, message: 'Initialization rejected' } });
      return;
    }
    await send({ jsonrpc: '2.0', id: hello.id, result: { protocol_version: PROTOCOL_VERSION, capabilities, ...context } });
    const request = await receive();
    if (typeof request.id !== 'string' || typeof request.method !== 'string') throw new Error('Request id and method required');
    const id = request.id;
    const abort = new AbortController();
    const call: Call = { request_id: id, context: Object.freeze({ ...context }), signal: abort.signal,
      emit: async (method, event) => {
        if (!eventMethods.has(method)) throw new RpcFailure(-32602, 'Unsupported event method');
        if (abort.signal.aborted) throw new RpcFailure(-32800, 'Invocation cancelled');
        await send({ jsonrpc: '2.0', method, params: { ...event, request_id: id } });
      } };
    const invocation = handler(request.method, (request.params ?? {}) as Frame, call);
    const cancellation = receive().then(() => { abort.abort(); throw new RpcFailure(-32800, 'Invocation cancelled'); },
      () => { abort.abort(); throw new RpcFailure(-32800, 'Host disconnected'); });
    try {
      const result = await Promise.race([invocation, cancellation]);
      await send({ jsonrpc: '2.0', id, result });
    } catch (error) {
      const known = error instanceof RpcFailure;
      await send({ jsonrpc: '2.0', id, error: { code: known ? error.code : -32603,
        message: known ? error.message.slice(0, 500) : 'Plugin handler failed' } });
    } finally { abort.abort(); }
  } finally { socket.end(); }
}
