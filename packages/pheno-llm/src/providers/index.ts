// LLM Provider Abstraction
// NEW module (no Rust equivalent)

export interface CompletionRequest {
  readonly model: string;
  readonly messages: Message[];
  readonly temperature?: number;
  readonly maxTokens?: number;
  readonly topP?: number;
  readonly stop?: string[];
}

export interface Message {
  readonly role: 'system' | 'user' | 'assistant' | 'tool';
  readonly content: string;
}

export interface CompletionResponse {
  readonly id: string;
  readonly content: string;
  readonly model: string;
  readonly usage: TokenUsage;
  readonly finishReason: 'stop' | 'length' | 'tool_calls';
}

export interface StreamResponse {
  readonly id: string;
  readonly stream: AsyncIterable<StreamChunk>;
}

export interface StreamChunk {
  readonly delta: string;
  readonly finishReason?: 'stop' | 'length' | 'tool_calls';
}

export interface TokenUsage {
  readonly promptTokens: number;
  readonly completionTokens: number;
  readonly totalTokens: number;
}

export interface LLMProvider {
  readonly name: string;
  complete(request: CompletionRequest): Promise<CompletionResponse>;
  stream(request: CompletionRequest): Promise<StreamResponse>;
}
