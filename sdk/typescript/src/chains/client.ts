import { Transport } from '../transport';
import { ValidationError } from '../errors';
import { ChainDefinition, ChainExecutionRequest, ChainExecutionResponse, ChainNode, ChainEdge } from './types';
import { asyncIterableToStream } from '../transport/stream';

/**
 * Client for chain execution
 */
export class ChainClient {
    private readonly transport: Transport;

    /**
     * Create a new chain client
     * @param transport Transport layer
     */
    constructor(transport: Transport) {
        this.transport = transport;
    }

    /**
     * Validate chain definition
     * @param chain Chain definition
     * @throws ValidationError if definition is invalid
     */
    private validateChainDefinition(chain: ChainDefinition): void {
        if (!chain.name) {
            throw new ValidationError('Chain name is required');
        }

        if (!chain.nodes || chain.nodes.length === 0) {
            throw new ValidationError('At least one node is required');
        }

        // Validate each node
        const nodeIds = new Set<string>();
        for (const node of chain.nodes) {
            if (!node.id) {
                throw new ValidationError('Node ID is required');
            }

            if (nodeIds.has(node.id)) {
                throw new ValidationError(`Duplicate node ID: ${node.id}`);
            }

            nodeIds.add(node.id);

            if (!node.type) {
                throw new ValidationError(`Node type is required for node ${node.id}`);
            }

            const validTypes = ['llm', 'function', 'conditional', 'loop', 'parallel', 'sequential'];
            if (!validTypes.includes(node.type)) {
                throw new ValidationError(`Invalid node type for node ${node.id}: ${node.type}`);
            }
        }

        // Validate edges
        if (chain.edges) {
            for (const edge of chain.edges) {
                if (!edge.source) {
                    throw new ValidationError('Edge source is required');
                }

                if (!edge.target) {
                    throw new ValidationError('Edge target is required');
                }

                if (!nodeIds.has(edge.source)) {
                    throw new ValidationError(`Edge source node ${edge.source} does not exist`);
                }

                if (!nodeIds.has(edge.target)) {
                    throw new ValidationError(`Edge target node ${edge.target} does not exist`);
                }

                if (edge.source === edge.target) {
                    throw new ValidationError(`Node ${edge.source} cannot connect to itself`);
                }
            }
        }
    }

    /**
     * Validate chain execution request
     * @param request Chain execution request
     * @throws ValidationError if request is invalid
     */
    private validateExecutionRequest(request: ChainExecutionRequest): void {
        if (!request.chain) {
            throw new ValidationError('Chain ID or definition is required');
        }

        if (typeof request.chain !== 'string') {
            this.validateChainDefinition(request.chain);
        }
    }

    /**
     * Create a new chain
     * @param chain Chain definition
     * @returns Promise resolving to the created chain
     */
    public async createChain(chain: ChainDefinition): Promise<ChainDefinition> {
        this.validateChainDefinition(chain);
        return this.transport.post<ChainDefinition>('/v1/chains', chain);
    }

    /**
     * Get a chain by ID
     * @param id Chain ID
     * @returns Promise resolving to the chain
     */
    public async getChain(id: string): Promise<ChainDefinition> {
        return this.transport.get<ChainDefinition>(`/v1/chains/${id}`);
    }

    /**
     * List all chains
     * @returns Promise resolving to an array of chains
     */
    public async listChains(): Promise<ChainDefinition[]> {
        return this.transport.get<ChainDefinition[]>('/v1/chains');
    }

    /**
     * Delete a chain
     * @param id Chain ID
     * @returns Promise resolving when the chain is deleted
     */
    public async deleteChain(id: string): Promise<void> {
        return this.transport.delete<void>(`/v1/chains/${id}`);
    }

    /**
     * Execute a chain
     * @param request Chain execution request
     * @returns Promise resolving to the chain execution response
     */
    public async executeChain(request: ChainExecutionRequest): Promise<ChainExecutionResponse> {
        this.validateExecutionRequest(request);

        if (request.stream) {
            throw new Error('Use executeChainStream for streaming responses');
        }

        return this.transport.post<ChainExecutionResponse>('/v1/chains/execute', request);
    }

    /**
     * Execute a chain with streaming response
     * @param request Chain execution request
     * @returns Promise resolving to a stream of chain execution chunks
     */
    public async executeChainStream(request: ChainExecutionRequest): Promise<ReadableStream<any>> {
        this.validateExecutionRequest(request);

        const streamRequest = {
            ...request,
            stream: true,
        };

        const stream = await this.transport.postStream('/v1/chains/execute', streamRequest);

        return asyncIterableToStream(stream);
    }
}