import { ChatClient } from '../client';
import { ValidationError } from '../../errors';
import { ChatCompletionOptions, ChatCompletionResponse, ChatCompletionChunk, ChatMessage, ChatRole } from '../types';

// Mock transport
const mockTransport = {
    post: jest.fn(),
    postStream: jest.fn(),
};

describe('ChatClient', () => {
    let client: ChatClient;

    beforeEach(() => {
        jest.clearAllMocks();
        client = new ChatClient(mockTransport as any);
    });

    describe('createCompletion', () => {
        it('should validate options before creating completion', async () => {
            const validOptions: ChatCompletionOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
            };

            const mockResponse: ChatCompletionResponse = {
                id: 'test-id',
                object: 'chat.completion',
                created: Date.now(),
                model: 'gpt-3.5-turbo',
                choices: [
                    {
                        index: 0,
                        message: {
                            role: 'assistant' as ChatRole,
                            content: 'Hello! How can I help you today?',
                        },
                        finish_reason: 'stop',
                    },
                ],
                usage: {
                    prompt_tokens: 10,
                    completion_tokens: 10,
                    total_tokens: 20,
                },
            };

            mockTransport.post.mockResolvedValue(mockResponse);

            const result = await client.createCompletion(validOptions);
            expect(result).toEqual(mockResponse);
            expect(mockTransport.post).toHaveBeenCalledWith('/v1/chat/completions', validOptions);
        });

        it('should throw validation error for missing model', async () => {
            const invalidOptions = {
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
            } as ChatCompletionOptions;

            await expect(client.createCompletion(invalidOptions)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for empty messages', async () => {
            const invalidOptions = {
                model: 'gpt-3.5-turbo',
                messages: [],
            } as ChatCompletionOptions;

            await expect(client.createCompletion(invalidOptions)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for invalid message role', async () => {
            const invalidOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'invalid' as any, content: 'Hello' },
                ],
            } as ChatCompletionOptions;

            await expect(client.createCompletion(invalidOptions)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for missing message content', async () => {
            const invalidOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole } as any,
                ],
            } as ChatCompletionOptions;

            await expect(client.createCompletion(invalidOptions)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for invalid temperature', async () => {
            const invalidOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
                temperature: 3, // Invalid: should be between 0 and 2
            } as ChatCompletionOptions;

            await expect(client.createCompletion(invalidOptions)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for invalid top_p', async () => {
            const invalidOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
                top_p: 2, // Invalid: should be between 0 and 1
            } as ChatCompletionOptions;

            await expect(client.createCompletion(invalidOptions)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw error when stream is true', async () => {
            const options = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
                stream: true,
            } as ChatCompletionOptions;

            await expect(client.createCompletion(options)).rejects.toThrow('Use createCompletionStream for streaming responses');
            expect(mockTransport.post).not.toHaveBeenCalled();
        });
    });

    describe('createCompletionStream', () => {
        it('should validate options before creating streaming completion', async () => {
            const validOptions: ChatCompletionOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
            };

            const mockChunk1: ChatCompletionChunk = {
                id: 'test-id',
                object: 'chat.completion.chunk',
                created: Date.now(),
                model: 'gpt-3.5-turbo',
                choices: [
                    {
                        index: 0,
                        delta: {
                            role: 'assistant' as ChatRole,
                            content: 'Hello',
                        },
                        finish_reason: null,
                    },
                ],
            };

            const mockChunk2: ChatCompletionChunk = {
                id: 'test-id',
                object: 'chat.completion.chunk',
                created: Date.now(),
                model: 'gpt-3.5-turbo',
                choices: [
                    {
                        index: 0,
                        delta: {
                            content: '! How can I help you today?',
                        },
                        finish_reason: 'stop',
                    },
                ],
            };

            const mockStream = (async function* () {
                yield mockChunk1;
                yield mockChunk2;
            })();

            mockTransport.postStream.mockResolvedValue(mockStream);

            // Mock the asyncIterableToStream function
            jest.mock('../../transport/stream', () => ({
                asyncIterableToStream: jest.fn().mockImplementation((stream) => stream),
            }));

            const stream = await client.createCompletionStream(validOptions);

            // Verify the stream works
            const chunks: ChatCompletionChunk[] = [];
            for await (const chunk of stream) {
                chunks.push(chunk);
            }

            expect(chunks).toEqual([mockChunk1, mockChunk2]);
            expect(mockTransport.postStream).toHaveBeenCalledWith('/v1/chat/completions', {
                ...validOptions,
                stream: true,
            });
        });

        it('should throw validation error for missing model', async () => {
            const invalidOptions = {
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
            } as ChatCompletionOptions;

            const streamPromise = client.createCompletionStream(invalidOptions);
            await expect(streamPromise.then(stream => stream.getReader().read())).rejects.toThrow(ValidationError);
            expect(mockTransport.postStream).not.toHaveBeenCalled();
        });

        it('should throw validation error for empty messages', async () => {
            const invalidOptions = {
                model: 'gpt-3.5-turbo',
                messages: [],
            } as ChatCompletionOptions;

            const streamPromise = client.createCompletionStream(invalidOptions);
            await expect(streamPromise.then(stream => stream.getReader().read())).rejects.toThrow(ValidationError);
            expect(mockTransport.postStream).not.toHaveBeenCalled();
        });
    });

    describe('complete', () => {
        it('should call createCompletion and return the message content', async () => {
            const messages: ChatMessage[] = [
                { role: 'user' as ChatRole, content: 'Hello' },
            ];
            const model = 'gpt-3.5-turbo';
            const options = { temperature: 0.7 };

            const mockResponse: ChatCompletionResponse = {
                id: 'test-id',
                object: 'chat.completion',
                created: Date.now(),
                model: 'gpt-3.5-turbo',
                choices: [
                    {
                        index: 0,
                        message: {
                            role: 'assistant' as ChatRole,
                            content: 'Hello! How can I help you today?',
                        },
                        finish_reason: 'stop',
                    },
                ],
                usage: {
                    prompt_tokens: 10,
                    completion_tokens: 10,
                    total_tokens: 20,
                },
            };

            mockTransport.post.mockResolvedValue(mockResponse);

            const result = await client.complete(messages, model, options);
            expect(result).toEqual('Hello! How can I help you today?');
            expect(mockTransport.post).toHaveBeenCalledWith('/v1/chat/completions', {
                messages,
                model,
                temperature: 0.7,
            });
        });

        it('should return empty string if no message content', async () => {
            const messages: ChatMessage[] = [
                { role: 'user' as ChatRole, content: 'Hello' },
            ];
            const model = 'gpt-3.5-turbo';

            const mockResponse: ChatCompletionResponse = {
                id: 'test-id',
                object: 'chat.completion',
                created: Date.now(),
                model: 'gpt-3.5-turbo',
                choices: [
                    {
                        index: 0,
                        message: {
                            role: 'assistant' as ChatRole,
                            content: null,
                            function_call: {
                                name: 'test_function',
                                arguments: '{}',
                            },
                        },
                        finish_reason: 'function_call',
                    },
                ],
                usage: {
                    prompt_tokens: 10,
                    completion_tokens: 10,
                    total_tokens: 20,
                },
            };

            mockTransport.post.mockResolvedValue(mockResponse);

            const result = await client.complete(messages, model);
            expect(result).toEqual('');
        });
    });

    describe('deprecated methods', () => {
        it('should call createCompletion from createChatCompletion', async () => {
            const options: ChatCompletionOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
            };

            const mockResponse: ChatCompletionResponse = {
                id: 'test-id',
                object: 'chat.completion',
                created: Date.now(),
                model: 'gpt-3.5-turbo',
                choices: [
                    {
                        index: 0,
                        message: {
                            role: 'assistant' as ChatRole,
                            content: 'Hello! How can I help you today?',
                        },
                        finish_reason: 'stop',
                    },
                ],
                usage: {
                    prompt_tokens: 10,
                    completion_tokens: 10,
                    total_tokens: 20,
                },
            };

            mockTransport.post.mockResolvedValue(mockResponse);

            const result = await client.createChatCompletion(options);
            expect(result).toEqual(mockResponse);
            expect(mockTransport.post).toHaveBeenCalledWith('/v1/chat/completions', options);
        });

        it('should call createCompletionStream from createChatCompletionStream', async () => {
            const options: ChatCompletionOptions = {
                model: 'gpt-3.5-turbo',
                messages: [
                    { role: 'user' as ChatRole, content: 'Hello' },
                ],
            };

            const mockStream = {} as ReadableStream<ChatCompletionChunk>;

            // Mock the createCompletionStream method
            jest.spyOn(client, 'createCompletionStream').mockResolvedValue(mockStream);

            const result = await client.createChatCompletionStream(options);
            expect(result).toBe(mockStream);
            expect(client.createCompletionStream).toHaveBeenCalledWith(options);
        });
    });
});