import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import { createParser } from 'eventsource-parser';
import { IntelliRouterConfig, RequestOptions, HttpMethod } from '../types';
import { Transport } from './base';
import {
    ApiError,
    AuthenticationError,
    AuthorizationError,
    NotFoundError,
    RateLimitError,
    ServerError,
    TimeoutError,
    ValidationError,
} from '../errors';

/**
 * HTTP transport implementation
 */
export class HttpTransport implements Transport {
    private readonly client: AxiosInstance;
    private readonly config: IntelliRouterConfig;

    /**
     * Create a new HTTP transport
     * @param config Client configuration
     */
    constructor(config: IntelliRouterConfig) {
        this.config = config;

        this.client = axios.create({
            baseURL: config.baseUrl || 'http://localhost:8000',
            timeout: config.timeout || 30000,
            headers: {
                'Content-Type': 'application/json',
                'Accept': 'application/json',
                ...(config.apiKey ? { 'Authorization': `Bearer ${config.apiKey}` } : {}),
                ...(config.defaultHeaders || {}),
            },
        });

        // Add request interceptor for retries
        this.client.interceptors.response.use(
            (response) => response,
            async (error) => {
                if (!error.config || !this.shouldRetry(error)) {
                    return Promise.reject(error);
                }

                const retryCount = error.config.retryCount || 0;
                const maxRetries = this.config.maxRetries || 3;

                if (retryCount >= maxRetries) {
                    return Promise.reject(error);
                }

                error.config.retryCount = retryCount + 1;

                // Exponential backoff
                const delay = Math.pow(2, retryCount) * 1000;
                await new Promise((resolve) => setTimeout(resolve, delay));

                return this.client(error.config);
            }
        );
    }

    /**
     * Determine if a request should be retried
     * @param error Axios error
     * @returns Whether the request should be retried
     */
    private shouldRetry(error: any): boolean {
        // Don't retry if we don't have a response
        if (!error.response) {
            return false;
        }

        // Don't retry client errors except for 429 (rate limit)
        if (error.response.status >= 400 && error.response.status < 500 && error.response.status !== 429) {
            return false;
        }

        return true;
    }

    /**
     * Handle an error response
     * @param error Axios error
     * @throws ApiError or a subclass
     */
    private handleError(error: any): never {
        if (error.response) {
            const { status, data } = error.response;
            const message = data?.error?.message || data?.message || error.message || 'Unknown error';

            switch (status) {
                case 400:
                    throw new ValidationError(message, { details: data });
                case 401:
                    throw new AuthenticationError(message, { details: data });
                case 403:
                    throw new AuthorizationError(message, { details: data });
                case 404:
                    throw new NotFoundError(message, { details: data });
                case 429:
                    throw new RateLimitError(message, { details: data });
                default:
                    if (status >= 500) {
                        throw new ServerError(message, { status, details: data });
                    }
                    throw new ApiError(message, { code: 'api_error', status, details: data });
            }
        } else if (error.code === 'ECONNABORTED') {
            throw new TimeoutError('Request timed out');
        } else {
            throw new ApiError(error.message || 'Unknown error', { code: 'network_error', status: 0 });
        }
    }

    /**
     * Send a request
     * @param options Request options
     * @returns Promise resolving to the response data
     */
    public async request<T>(options: RequestOptions): Promise<T> {
        try {
            const config: AxiosRequestConfig = {
                method: options.method,
                url: options.path,
                params: options.params,
                data: options.body,
                headers: options.headers,
            };

            const response = await this.client.request<T>(config);
            return response.data;
        } catch (error) {
            this.handleError(error);
        }
    }

    /**
     * Send a request with streaming response
     * @param options Request options
     * @returns Promise resolving to an async iterable of response chunks
     */
    public async requestStream(options: RequestOptions): Promise<AsyncIterable<unknown>> {
        try {
            const config: AxiosRequestConfig = {
                method: options.method,
                url: options.path,
                params: options.params,
                data: options.body,
                headers: {
                    ...options.headers,
                    'Accept': 'text/event-stream',
                },
                responseType: 'stream',
            };

            const response = await this.client.request(config);

            return this.parseEventStream(response);
        } catch (error) {
            this.handleError(error);
        }
    }

    /**
     * Parse an event stream response
     * @param response Axios response
     * @returns Async iterable of parsed events
     */
    private async *parseEventStream(response: AxiosResponse): AsyncIterable<unknown> {
        const parser = createParser((event) => {
            if (event.type === 'event' && event.data) {
                if (event.data === '[DONE]') {
                    return;
                }

                try {
                    const parsedData = JSON.parse(event.data);
                    return parsedData;
                } catch (e) {
                    return event.data;
                }
            }
        });

        const stream = response.data;

        for await (const chunk of stream) {
            const str = chunk.toString();
            const parsedEvents = parser.feed(str);

            for (const event of parsedEvents) {
                if (event) {
                    yield event;
                }
            }
        }
    }

    /**
     * Send a GET request
     * @param path URL path
     * @param params Query parameters
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    public async get<T>(path: string, params?: Record<string, string>, headers?: Record<string, string>): Promise<T> {
        return this.request<T>({
            method: 'GET',
            path,
            params,
            headers,
        });
    }

    /**
     * Send a POST request
     * @param path URL path
     * @param body Request body
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    public async post<T>(path: string, body?: unknown, headers?: Record<string, string>): Promise<T> {
        return this.request<T>({
            method: 'POST',
            path,
            body,
            headers,
        });
    }

    /**
     * Send a PUT request
     * @param path URL path
     * @param body Request body
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    public async put<T>(path: string, body?: unknown, headers?: Record<string, string>): Promise<T> {
        return this.request<T>({
            method: 'PUT',
            path,
            body,
            headers,
        });
    }

    /**
     * Send a DELETE request
     * @param path URL path
     * @param headers Request headers
     * @returns Promise resolving to the response data
     */
    public async delete<T>(path: string, headers?: Record<string, string>): Promise<T> {
        return this.request<T>({
            method: 'DELETE',
            path,
            headers,
        });
    }

    /**
     * Send a POST request with streaming response
     * @param path URL path
     * @param body Request body
     * @param headers Request headers
     * @returns Promise resolving to an async iterable of response chunks
     */
    public async postStream(path: string, body?: unknown, headers?: Record<string, string>): Promise<AsyncIterable<unknown>> {
        return this.requestStream({
            method: 'POST',
            path,
            body,
            headers,
            stream: true,
        });
    }
}