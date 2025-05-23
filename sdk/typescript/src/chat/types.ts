/**
 * Role of a chat message
 */
export type ChatRole = 'system' | 'user' | 'assistant' | 'function' | 'tool';

/**
 * A single message in a chat conversation
 */
export interface ChatMessage {
    /**
     * Role of the message sender
     */
    role: ChatRole;

    /**
     * Content of the message
     */
    content: string | null;

    /**
     * Optional name of the sender
     */
    name?: string;

    /**
     * Optional function call
     */
    function_call?: {
        name: string;
        arguments: string;
    };

    /**
     * Optional tool calls
     */
    tool_calls?: Array<{
        id: string;
        type: 'function';
        function: {
            name: string;
            arguments: string;
        };
    }>;
}

/**
 * Options for chat completions
 */
export interface ChatCompletionOptions {
    /**
     * Model to use for completions
     */
    model: string;

    /**
     * Messages in the conversation
     */
    messages: ChatMessage[];

    /**
     * Sampling temperature
     * @default 1.0
     */
    temperature?: number;

    /**
     * Top-p sampling
     * @default 1.0
     */
    top_p?: number;

    /**
     * Number of completions to generate
     * @default 1
     */
    n?: number;

    /**
     * Maximum number of tokens to generate
     */
    max_tokens?: number;

    /**
     * Sequences where the API will stop generating further tokens
     */
    stop?: string | string[];

    /**
     * Presence penalty
     * @default 0.0
     */
    presence_penalty?: number;

    /**
     * Frequency penalty
     * @default 0.0
     */
    frequency_penalty?: number;

    /**
     * Logit bias
     */
    logit_bias?: Record<string, number>;

    /**
     * User identifier
     */
    user?: string;

    /**
     * Whether to stream the response
     * @default false
     */
    stream?: boolean;

    /**
     * Functions that the model may generate JSON inputs for
     */
    functions?: Array<{
        name: string;
        description?: string;
        parameters?: Record<string, unknown>;
    }>;

    /**
     * Controls how the model responds to function calls
     */
    function_call?: 'none' | 'auto' | { name: string };

    /**
     * Available tools the model may call
     */
    tools?: Array<{
        type: 'function';
        function: {
            name: string;
            description?: string;
            parameters?: Record<string, unknown>;
        };
    }>;

    /**
     * Controls which (if any) tool is called by the model
     */
    tool_choice?: 'none' | 'auto' | { type: 'function'; function: { name: string } };
}

/**
 * Response from a chat completion request
 */
export interface ChatCompletionResponse {
    /**
     * Unique identifier for the completion
     */
    id: string;

    /**
     * Object type
     */
    object: string;

    /**
     * Creation timestamp
     */
    created: number;

    /**
     * Model used for the completion
     */
    model: string;

    /**
     * Completion choices
     */
    choices: Array<{
        /**
         * Index of the choice
         */
        index: number;

        /**
         * Message generated by the model
         */
        message: ChatMessage;

        /**
         * Reason the completion finished
         */
        finish_reason: 'stop' | 'length' | 'function_call' | 'tool_calls' | 'content_filter';
    }>;

    /**
     * Token usage information
     */
    usage: {
        /**
         * Number of tokens in the prompt
         */
        prompt_tokens: number;

        /**
         * Number of tokens in the completion
         */
        completion_tokens: number;

        /**
         * Total number of tokens used
         */
        total_tokens: number;
    };
}

/**
 * A chunk of a streaming chat completion response
 */
export interface ChatCompletionChunk {
    /**
     * Unique identifier for the chunk
     */
    id: string;

    /**
     * Object type
     */
    object: string;

    /**
     * Creation timestamp
     */
    created: number;

    /**
     * Model used for the completion
     */
    model: string;

    /**
     * Chunk choices
     */
    choices: Array<{
        /**
         * Index of the choice
         */
        index: number;

        /**
         * Delta of the message
         */
        delta: Partial<ChatMessage>;

        /**
         * Reason the completion finished
         */
        finish_reason: 'stop' | 'length' | 'function_call' | 'tool_calls' | 'content_filter' | null;
    }>;
}

/**
 * @deprecated Use ChatCompletionOptions instead
 */
export interface ChatCompletionRequest extends ChatCompletionOptions { }

/**
 * @deprecated Use the choices array directly
 */
export interface ChatCompletionChoice {
    /**
     * Index of the choice
     */
    index: number;

    /**
     * Generated message
     */
    message: ChatMessage;

    /**
     * Reason for finishing
     */
    finish_reason: 'stop' | 'length' | 'function_call' | 'tool_calls' | 'content_filter';
}