/**
 * Chain node type
 */
export type ChainNodeType = 'llm' | 'function' | 'conditional' | 'loop' | 'parallel' | 'sequential';

/**
 * Base chain node
 */
export interface ChainNode {
    /**
     * Node ID
     */
    id: string;

    /**
     * Node type
     */
    type: ChainNodeType;

    /**
     * Node name
     */
    name?: string;

    /**
     * Node description
     */
    description?: string;

    /**
     * Node inputs
     */
    inputs?: Record<string, any>;

    /**
     * Node outputs
     */
    outputs?: string[];
}

/**
 * LLM chain node
 */
export interface LlmChainNode extends ChainNode {
    /**
     * Node type
     */
    type: 'llm';

    /**
     * Model to use
     */
    model: string;

    /**
     * Prompt template
     */
    prompt: string;

    /**
     * Sampling temperature
     */
    temperature?: number;

    /**
     * Maximum number of tokens to generate
     */
    max_tokens?: number;
}

/**
 * Function chain node
 */
export interface FunctionChainNode extends ChainNode {
    /**
     * Node type
     */
    type: 'function';

    /**
     * Function name
     */
    function: string;

    /**
     * Function arguments
     */
    arguments?: Record<string, any>;
}

/**
 * Chain definition
 */
export interface ChainDefinition {
    /**
     * Chain ID
     */
    id: string;

    /**
     * Chain name
     */
    name: string;

    /**
     * Chain description
     */
    description?: string;

    /**
     * Chain version
     */
    version?: string;

    /**
     * Chain nodes
     */
    nodes: ChainNode[];

    /**
     * Chain edges
     */
    edges: ChainEdge[];
}

/**
 * Chain edge
 */
export interface ChainEdge {
    /**
     * Source node ID
     */
    source: string;

    /**
     * Target node ID
     */
    target: string;

    /**
     * Source output name
     */
    sourceOutput?: string;

    /**
     * Target input name
     */
    targetInput?: string;
}

/**
 * Chain execution request
 */
export interface ChainExecutionRequest {
    /**
     * Chain ID or definition
     */
    chain: string | ChainDefinition;

    /**
     * Input values
     */
    inputs?: Record<string, any>;

    /**
     * Whether to stream the response
     */
    stream?: boolean;
}

/**
 * Chain execution response
 */
export interface ChainExecutionResponse {
    /**
     * Execution ID
     */
    id: string;

    /**
     * Chain ID
     */
    chainId: string;

    /**
     * Execution status
     */
    status: 'success' | 'error' | 'running';

    /**
     * Output values
     */
    outputs: Record<string, any>;

    /**
     * Error message (if status is 'error')
     */
    error?: string;

    /**
     * Execution metrics
     */
    metrics?: {
        /**
         * Start time
         */
        startTime: string;

        /**
         * End time
         */
        endTime: string;

        /**
         * Duration in milliseconds
         */
        duration: number;

        /**
         * Token usage
         */
        tokens?: {
            /**
             * Number of prompt tokens
             */
            prompt: number;

            /**
             * Number of completion tokens
             */
            completion: number;

            /**
             * Total number of tokens
             */
            total: number;
        };
    };
}