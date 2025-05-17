/**
 * Configuration options for the IntelliRouter client
 */
export interface IntelliRouterConfig {
    /**
     * Base URL for the IntelliRouter API
     * @default "http://localhost:8000"
     */
    baseUrl?: string;

    /**
     * API key for authentication
     */
    apiKey?: string;

    /**
     * Request timeout in milliseconds
     * @default 30000
     */
    timeout?: number;

    /**
     * Maximum number of retries for failed requests
     * @default 3
     */
    maxRetries?: number;

    /**
     * Default headers to include with every request
     */
    defaultHeaders?: Record<string, string>;

    /**
     * Transport configuration
     */
    transport?: TransportConfig;
}

/**
 * Configuration for the transport layer
 */
export interface TransportConfig {
    /**
     * Transport type
     * @default "http"
     */
    type?: 'http' | 'websocket';

    /**
     * Keep-alive timeout in milliseconds
     * @default 60000
     */
    keepAliveTimeout?: number;

    /**
     * Whether to use HTTPS
     * @default true
     */
    secure?: boolean;
}

/**
 * HTTP methods supported by the transport layer
 */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE' | 'PATCH';

/**
 * Options for HTTP requests
 */
export interface RequestOptions {
    /**
     * HTTP method
     */
    method: HttpMethod;

    /**
     * URL path
     */
    path: string;

    /**
     * Query parameters
     */
    params?: Record<string, string>;

    /**
     * Request body
     */
    body?: unknown;

    /**
     * Request headers
     */
    headers?: Record<string, string>;

    /**
     * Whether to stream the response
     */
    stream?: boolean;
}