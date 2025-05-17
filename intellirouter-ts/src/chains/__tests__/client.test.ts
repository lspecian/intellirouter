import { ChainClient } from '../client';
import { ValidationError } from '../../errors';
import { ChainDefinition, ChainExecutionRequest, LlmChainNode, FunctionChainNode } from '../types';

// Mock transport
const mockTransport = {
    post: jest.fn(),
    get: jest.fn(),
    delete: jest.fn(),
    postStream: jest.fn(),
};

describe('ChainClient', () => {
    let client: ChainClient;

    beforeEach(() => {
        jest.clearAllMocks();
        client = new ChainClient(mockTransport as any);
    });

    describe('createChain', () => {
        it('should validate chain definition before creating', async () => {
            const llmNode: LlmChainNode = {
                id: 'node1',
                type: 'llm',
                name: 'LLM Node',
                model: 'gpt-4',
                prompt: 'Hello, world!',
            };

            const validChain: ChainDefinition = {
                id: 'test-chain',
                name: 'Test Chain',
                nodes: [llmNode],
                edges: [],
            };

            mockTransport.post.mockResolvedValue(validChain);

            const result = await client.createChain(validChain);
            expect(result).toEqual(validChain);
            expect(mockTransport.post).toHaveBeenCalledWith('/v1/chains', validChain);
        });

        it('should throw validation error for invalid chain definition', async () => {
            const invalidChain = {
                id: 'test-chain',
                // Missing name
                nodes: [],
                edges: [],
            } as unknown as ChainDefinition;

            await expect(client.createChain(invalidChain)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for empty nodes', async () => {
            const invalidChain = {
                id: 'test-chain',
                name: 'Test Chain',
                nodes: [], // Empty nodes
                edges: [],
            } as ChainDefinition;

            await expect(client.createChain(invalidChain)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for invalid node type', async () => {
            const invalidChain: ChainDefinition = {
                id: 'test-chain',
                name: 'Test Chain',
                nodes: [
                    {
                        id: 'node1',
                        type: 'invalid-type' as any,
                        name: 'Invalid Node',
                    },
                ],
                edges: [],
            };

            await expect(client.createChain(invalidChain)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for duplicate node IDs', async () => {
            const llmNode: LlmChainNode = {
                id: 'node1',
                type: 'llm',
                name: 'LLM Node 1',
                model: 'gpt-4',
                prompt: 'Hello, world!',
            };

            const functionNode: FunctionChainNode = {
                id: 'node1', // Duplicate ID
                type: 'function',
                name: 'Function Node',
                function: 'testFunction',
            };

            const invalidChain: ChainDefinition = {
                id: 'test-chain',
                name: 'Test Chain',
                nodes: [llmNode, functionNode],
                edges: [],
            };

            await expect(client.createChain(invalidChain)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw validation error for invalid edge', async () => {
            const llmNode: LlmChainNode = {
                id: 'node1',
                type: 'llm',
                name: 'LLM Node',
                model: 'gpt-4',
                prompt: 'Hello, world!',
            };

            const invalidChain: ChainDefinition = {
                id: 'test-chain',
                name: 'Test Chain',
                nodes: [llmNode],
                edges: [
                    {
                        source: 'node1',
                        target: 'node2', // Non-existent target
                    },
                ],
            };

            await expect(client.createChain(invalidChain)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });
    });

    describe('getChain', () => {
        it('should get a chain by ID', async () => {
            const chainId = 'test-chain';
            const llmNode: LlmChainNode = {
                id: 'node1',
                type: 'llm',
                name: 'LLM Node',
                model: 'gpt-4',
                prompt: 'Hello, world!',
            };

            const mockChain: ChainDefinition = {
                id: chainId,
                name: 'Test Chain',
                nodes: [llmNode],
                edges: [],
            };

            mockTransport.get.mockResolvedValue(mockChain);

            const result = await client.getChain(chainId);
            expect(result).toEqual(mockChain);
            expect(mockTransport.get).toHaveBeenCalledWith(`/v1/chains/${chainId}`);
        });
    });

    describe('listChains', () => {
        it('should list all chains', async () => {
            const mockChains: ChainDefinition[] = [
                {
                    id: 'chain1',
                    name: 'Chain 1',
                    nodes: [],
                    edges: [],
                },
                {
                    id: 'chain2',
                    name: 'Chain 2',
                    nodes: [],
                    edges: [],
                },
            ];

            mockTransport.get.mockResolvedValue(mockChains);

            const result = await client.listChains();
            expect(result).toEqual(mockChains);
            expect(mockTransport.get).toHaveBeenCalledWith('/v1/chains');
        });
    });

    describe('deleteChain', () => {
        it('should delete a chain by ID', async () => {
            const chainId = 'test-chain';

            mockTransport.delete.mockResolvedValue(undefined);

            await client.deleteChain(chainId);
            expect(mockTransport.delete).toHaveBeenCalledWith(`/v1/chains/${chainId}`);
        });
    });

    describe('executeChain', () => {
        it('should execute a chain with valid request', async () => {
            const request: ChainExecutionRequest = {
                chain: 'test-chain',
                inputs: {
                    input1: 'value1',
                },
            };

            const mockResponse = {
                id: 'execution-id',
                chainId: 'test-chain',
                status: 'success',
                outputs: {
                    output1: 'result1',
                },
            };

            mockTransport.post.mockResolvedValue(mockResponse);

            const result = await client.executeChain(request);
            expect(result).toEqual(mockResponse);
            expect(mockTransport.post).toHaveBeenCalledWith('/v1/chains/execute', request);
        });

        it('should throw validation error for invalid request', async () => {
            const invalidRequest = {} as ChainExecutionRequest;

            await expect(client.executeChain(invalidRequest)).rejects.toThrow(ValidationError);
            expect(mockTransport.post).not.toHaveBeenCalled();
        });

        it('should throw error when stream is true', async () => {
            const request: ChainExecutionRequest = {
                chain: 'test-chain',
                inputs: {
                    input1: 'value1',
                },
                stream: true,
            };

            await expect(client.executeChain(request)).rejects.toThrow('Use executeChainStream for streaming responses');
            expect(mockTransport.post).not.toHaveBeenCalled();
        });
    });

    describe('executeChainStream', () => {
        it('should execute a chain with streaming response', async () => {
            const request: ChainExecutionRequest = {
                chain: 'test-chain',
                inputs: {
                    input1: 'value1',
                },
            };

            const mockStream = {} as AsyncIterable<unknown>;
            const mockReadableStream = {} as ReadableStream<unknown>;

            mockTransport.postStream.mockResolvedValue(mockStream);

            // Mock the asyncIterableToStream function
            jest.mock('../../transport/stream', () => ({
                asyncIterableToStream: jest.fn().mockReturnValue(mockReadableStream),
            }));

            const result = await client.executeChainStream(request);
            expect(result).toBe(mockReadableStream);
            expect(mockTransport.postStream).toHaveBeenCalledWith('/v1/chains/execute', {
                ...request,
                stream: true,
            });
        });

        it('should throw validation error for invalid request', async () => {
            const invalidRequest = {} as ChainExecutionRequest;

            await expect(client.executeChainStream(invalidRequest)).rejects.toThrow(ValidationError);
            expect(mockTransport.postStream).not.toHaveBeenCalled();
        });
    });
});