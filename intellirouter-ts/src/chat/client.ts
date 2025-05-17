import { Transport } from '../transport';
import { ValidationError } from '../errors';
import {
    ChatCompletionOptions,
    ChatCompletionResponse,
    ChatCompletionChunk,
    ChatMessage,
    ChatRole,
    ChatCompletionRequest
} from './types';
import { asyncIterableToStream } from '../transport/stream';

/**
 * Client for chat completions
 */
export class ChatClient {
    private readonly transport: Transport;

    /**
     * Create a new chat client
     * @param transport Transport layer
     */
    constructor(transport: Transport) {
        this.transport = transport;
    }

    /**
     * Validate chat completion options
     * @param options Chat completion options
     * @throws ValidationError if options are invalid
     */
    private validateOptions(options: ChatCompletionOptions): void {
        if (!options.model) {
            throw new ValidationError('Model is required');
        }

        if (!options.messages || !Array.isArray(options.messages) || options.messages.length === 0) {
            throw new ValidationError('At least one message is required');
        }

        // Validate each message
        for (const message of options.messages) {
            if (!message.role) {
                throw new ValidationError('Message role is required');
            }

            const validRoles: ChatRole[] = ['system', 'user', 'assistant', 'function', 'tool'];
            if (!validRoles.includes(message.role as ChatRole)) {
                throw new ValidationError(`Invalid message role: ${message.role}`);
            }

            if (message.content === undefined && message.content !== null && !message.function_call && !message.tool_calls) {
                throw new ValidationError('Message must have content, function_call, or tool_calls');
            }
        }

        // Validate numeric parameters
        if (options.temperature !== undefined && (options.temperature < 0 || options.temperature > 2)) {
            throw new ValidationError('Temperature must be between 0 and 2');
        }

        if (options.top_p !== undefined && (options.top_p < 0 || options.top_p > 1)) {
            throw new ValidationError('Top-p must be between 0 and 1');
        }

        if (options.n !== undefined && (options.n < 1 || options.n > 10)) {
            throw new ValidationError('N must be between 1 and 10');
        }

        if (options.presence_penalty !== undefined && (options.presence_penalty < -2 || options.presence_penalty > 2)) {
            throw new ValidationError('Presence penalty must be between -2 and 2');
        }

        if (options.frequency_penalty !== undefined && (options.frequency_penalty < -2 || options.frequency_penalty > 2)) {
            throw new ValidationError('Frequency penalty must be between -2 and 2');
        }
    }

    /**
     * Create a chat completion
     * @param options Chat completion options
     * @returns Promise resolving to a chat completion response
     */
    public async createCompletion(options: ChatCompletionOptions): Promise<ChatCompletionResponse> {
        this.validateOptions(options);

        if (options.stream) {
            throw new Error('Use createCompletionStream for streaming responses');
        }

        return this.transport.post<ChatCompletionResponse>('/v1/chat/completions', options);
    }

    /**
     * Create a streaming chat completion
     * @param options Chat completion options
     * @returns Promise resolving to a stream of chat completion chunks
     */
    public async createCompletionStream(options: ChatCompletionOptions): Promise<ReadableStream<ChatCompletionChunk>> {
        this.validateOptions(options);

        const streamOptions = { ...options, stream: true };

        const stream = await this.transport.postStream('/v1/chat/completions', streamOptions);

        // Use type assertion to convert the unknown stream to ChatCompletionChunk stream
        return asyncIterableToStream<ChatCompletionChunk>(stream as AsyncIterable<ChatCompletionChunk>);
    }

    /**
     * Create a simple completion with just messages
     * @param messages Chat messages
     * @param model Model to use
     * @param options Additional options
     * @returns Promise resolving to the generated message
     */
    public async complete(
        messages: ChatMessage[],
        model: string,
        options: Omit<ChatCompletionOptions, 'messages' | 'model'> = {}
    ): Promise<string> {
        const response = await this.createCompletion({
            messages,
            model,
            ...options,
        });

        return response.choices[0]?.message?.content || '';
    }

    /**
     * @deprecated Use createCompletion instead
     */
    public async createChatCompletion(request: ChatCompletionRequest): Promise<ChatCompletionResponse> {
        return this.createCompletion(request);
    }

    /**
     * @deprecated Use createCompletionStream instead
     */
    public async createChatCompletionStream(request: ChatCompletionRequest): Promise<ReadableStream<ChatCompletionChunk>> {
        return this.createCompletionStream(request);
    }
}