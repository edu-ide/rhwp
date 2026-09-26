import {
  EMBED_PROTOCOL_VERSION,
  EMBED_CAPABILITIES,
  isConnectAttempt,
  isConnectMessage,
  isRequestAttempt,
  isRequestEnvelope,
  isUsableParentOrigin,
  type EmbedResponseEnvelope,
} from './protocol.ts';
import { OFFICE_EMBED_METHODS, routeEmbedRequest, type EmbedRpcHandlers } from './rpc-router.ts';
import { isDocumentAgentError } from '../document-agent/types.ts';

interface EmbedRuntimeOptions {
  hostWindow: Window;
  parentWindow: Window;
  handlers: EmbedRpcHandlers;
  subscribeDocumentChanged?: (listener: (payload: unknown) => void) => () => void;
  subscribeRealtimeOperation?: (listener: (payload: unknown) => void) => () => void;
}

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function postPortResponse(port: MessagePort, response: EmbedResponseEnvelope): void {
  if (!(response.result instanceof Uint8Array)) {
    port.postMessage(response);
    return;
  }
  const result = response.result.slice();
  port.postMessage({ ...response, result }, [result.buffer]);
}

function releasePort(port: MessagePort | undefined): void {
  if (!port) return;
  port.onmessage = null;
  port.close();
}

function releasePorts(ports: readonly MessagePort[]): void {
  for (const port of ports) releasePort(port);
}

function bindPort(
  port: MessagePort,
  sessionId: string,
  clientCapabilities: readonly string[],
  handlers: EmbedRpcHandlers,
  subscribeDocumentChanged?: (listener: (payload: unknown) => void) => () => void,
  subscribeRealtimeOperation?: (listener: (payload: unknown) => void) => () => void,
): () => void {
  port.onmessage = async ({ data }) => {
    if (!isRequestAttempt(data, sessionId)) return;
    const response: EmbedResponseEnvelope = {
      type: 'rhwp-response', version: EMBED_PROTOCOL_VERSION, sessionId, id: data.id,
    };
    if (!isRequestEnvelope(data, sessionId)) {
      response.error = typeof data.version === 'number'
        && Number.isSafeInteger(data.version)
        && data.version !== EMBED_PROTOCOL_VERSION
        ? {
            code: 'UNSUPPORTED_VERSION',
            message: `Unsupported embed protocol version: ${data.version}`,
            supportedVersions: [EMBED_PROTOCOL_VERSION],
          }
        : { code: 'INVALID_REQUEST', message: 'Invalid embed request.' };
      postPortResponse(port, response);
      return;
    }
    const requiredCapability = OFFICE_EMBED_METHODS.has(data.method) ? 'office-realtime-v1' : {
      getDocumentState: 'document-state-v1',
      getSelectionContext: 'selection-context-v1',
      applyTextCommand: 'document-agent-command-v1',
      revertTextCommand: 'document-agent-command-v1',
      focusTarget: 'target-navigation-v1',
    }[data.method];
    if (requiredCapability && !clientCapabilities.includes(requiredCapability)) {
      response.error = {
        code: 'UNSUPPORTED_CAPABILITY',
        message: `${requiredCapability} was not negotiated by the client.`,
      };
      postPortResponse(port, response);
      return;
    }
    try {
      response.result = await routeEmbedRequest(data.method, data.params, handlers);
    } catch (error) {
      const documentAgentError = isDocumentAgentError(error) ? error : null;
      response.error = {
        code: documentAgentError?.code ?? 'RPC_ERROR',
        message: errorText(error),
        ...(typeof documentAgentError?.recovered === 'boolean'
          ? { recovered: documentAgentError.recovered }
          : {}),
      };
    }
    postPortResponse(port, response);
  };
  port.start();
  port.postMessage({
    type: 'rhwp-connected', version: EMBED_PROTOCOL_VERSION, sessionId,
    capabilities: EMBED_CAPABILITIES,
  });
  const subscribe = (event: string, source?: (listener: (payload: unknown) => void) => () => void) => source?.(payload => {
    port.postMessage({ type: 'rhwp-event', version: EMBED_PROTOCOL_VERSION, sessionId, event, payload });
  });
  const offDocument = subscribe('documentChanged', subscribeDocumentChanged);
  const offOperation = subscribe('operation', subscribeRealtimeOperation);
  return () => { offDocument?.(); offOperation?.(); };

}

function rejectConnect(port: MessagePort, attempt: { version: number; sessionId: string }): void {
  port.start();
  port.postMessage({
    type: 'rhwp-connect-error',
    version: EMBED_PROTOCOL_VERSION,
    sessionId: attempt.sessionId,
    error: {
      code: attempt.version === EMBED_PROTOCOL_VERSION
        ? 'UNSUPPORTED_CAPABILITY'
        : 'UNSUPPORTED_VERSION',
      message: attempt.version === EMBED_PROTOCOL_VERSION
        ? '필수 embed capability를 지원하지 않습니다.'
        : `지원하지 않는 embed protocol version: ${attempt.version}`,
      supportedVersions: [EMBED_PROTOCOL_VERSION],
    },
  });
  releasePort(port);
}

async function handleLegacy(
  event: MessageEvent,
  handlers: EmbedRpcHandlers,
): Promise<void> {
  const message = event.data;
  const isHwpctl = message?.type === 'hwpctl-load' && message.data;
  if (!isHwpctl && (message?.type !== 'rhwp-request' || !message.method)) return;
  const method = isHwpctl ? 'loadFile' : message.method;
  const params = isHwpctl ? message : message.params;
  const response: Record<string, unknown> = { type: 'rhwp-response', id: message.id };
  try {
    if (method === 'applyTextCommand' || method === 'revertTextCommand') {
      throw new Error('Legacy embed transport cannot execute document mutations.');
    }
    const result = await routeEmbedRequest(method, params, handlers, true);
    response.result = result instanceof Uint8Array ? Array.from(result) : result;
  } catch (error) {
    response.error = errorText(error);
  }
  (event.source as WindowProxy | null)?.postMessage(response, { targetOrigin: event.origin });
}

export function installEmbedRuntime(options: EmbedRuntimeOptions): () => void {
  const ports = new Set<MessagePort>();
  const isTopLevelSameWindow = options.parentWindow === options.hostWindow;
  let binding: {
    origin: string;
    sessionId: string;
    port: MessagePort;
    officeRealtime: boolean;
    offDocumentChanged: () => void;
  } | null = null;
  // Office's injected shortcut handler dispatches locally. Keep save and edit
  // events on the same bound port so save snapshots cannot overtake prior edits.
  const onHostCommand = (event: Event) => {
    const detail: unknown = (event as CustomEvent).detail;
    if (!binding?.officeRealtime || !detail || typeof detail !== 'object'
      || !('command' in detail) || detail.command !== 'save') return;
    binding.port.postMessage({
      type: 'rhwp-event', version: EMBED_PROTOCOL_VERSION, sessionId: binding.sessionId,
      event: 'host-command', payload: { command: 'save' },
    });
  };
  const onMessage = (event: MessageEvent) => {
    const transferredPorts = Array.from(event.ports);
    const isTopLevelLegacyRequest = isTopLevelSameWindow
      && event.origin !== 'null' && event.origin !== '*'
      && event.data?.type === 'rhwp-request';
    const location = options.hostWindow.location;
    const localOrigin = location && ['tauri:', 'asset:', 'app:', 'alhangeul-studio:'].includes(location.protocol)
      ? `${location.protocol}//${location.host}` : null;
    const isSameLocalOrigin = !isTopLevelSameWindow && localOrigin !== null && event.origin === localOrigin;
    const isOpaquePortHandshake = !isTopLevelSameWindow && event.origin === 'null'
      && transferredPorts.length === 1 && isConnectMessage(event.data);
    if (
      event.source !== options.parentWindow
      || (!isUsableParentOrigin(event.origin) && !isTopLevelLegacyRequest && !isOpaquePortHandshake && !isSameLocalOrigin)
    ) {
      releasePorts(transferredPorts);
      return;
    }
    const port = event.data?.type === 'rhwp-connect' ? transferredPorts.shift() : undefined;
    releasePorts(transferredPorts);
    if (binding && event.origin !== binding.origin) {
      releasePort(port);
      return;
    }
    if (port) {
      if (!isConnectAttempt(event.data)) {
        releasePort(port);
        return;
      }
      if (!isConnectMessage(event.data)) {
        rejectConnect(port, event.data);
        return;
      }
      if (binding) {
        if (event.data.sessionId !== binding.sessionId) {
          releasePort(port);
          return;
        }
        // A retry replaces only the same parent's same-origin, same-session channel.
        // The first acknowledgement may have raced a client's bounded retry timer.
        binding.offDocumentChanged();
        releasePort(binding.port);
        ports.delete(binding.port);
      }
      ports.add(port);
      const offDocumentChanged = bindPort(
        port,
        event.data.sessionId,
        event.data.capabilities,
        options.handlers,
        event.data.capabilities.includes('document-change-events-v1')
          ? options.subscribeDocumentChanged
          : undefined,
        event.data.capabilities.includes('office-realtime-v1') ? options.subscribeRealtimeOperation : undefined,
      );
      binding = { origin: event.origin, sessionId: event.data.sessionId, port,
        officeRealtime: event.data.capabilities.includes('office-realtime-v1'), offDocumentChanged };
      return;
    }
    if (binding) return;
    void handleLegacy(event, options.handlers);
  };
  options.hostWindow.addEventListener('message', onMessage);
  options.hostWindow.addEventListener('rhwp-host-command', onHostCommand);
  return () => {
    options.hostWindow.removeEventListener('message', onMessage);
    options.hostWindow.removeEventListener('rhwp-host-command', onHostCommand);
    binding?.offDocumentChanged();
    for (const port of ports) releasePort(port);
    ports.clear();
    binding = null;
  };
}
