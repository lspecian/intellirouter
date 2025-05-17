import { RequestOptions } from '../types';

/**
 * Base interface for transport implementations
 */
export interface Transport {
    /**
     * Send a request
     * @param options Request options
     * @returns Promise resolving to the response data
     */
    request<T>(options: RequestOptions): Promise<T>;

    /**
     * Send a request with streaming response
     * @param options Request options
     * @returns Promise resolving to an async iterable of response chunks
     */
    requestStream(options: RequestOptions): Promise<AsyncIterable<unknown>>;

    /**
     * Send a GET request
     * @param path URL path
     * @param params Query parameters
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    get<T>(path: string, params?: Record<string, string>, headers?: Record<string, string>): Promise<T>;

    /**
     * Send a POST request
     * @param path URL path
     * @param body Request body
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    post<T>(path: string, body?: unknown, headers?: Record<string, string>): Promise<T>;

    /**
     * Send a PUT request
     * @param path URL path
     * @param body Request body
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    put<T>(path: string, body?: unknown, headers?: Record<string, string>): Promise<T>;

    /**
     * Send a DELETE request
     * @param path URL path
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    delete<T>(path: string, headers?: Record<string, string>): Promise<T>;

    /**
     * Send a POST request with streaming response
     * @param path URL path
     * @param body Request body
     * @param headers Request headers
     * @returns Promise resolving to an async iterable of response chunks
     */
    postStream(path: string, body?: unknown, headers?: Record<string, string>): Promise<AsyncIterable<unknown>>;
}