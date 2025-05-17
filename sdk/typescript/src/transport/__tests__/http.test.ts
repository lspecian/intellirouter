import axios from 'axios';
import { HttpTransport } from '../http';
import {
    ApiError,
    AuthenticationError,
    AuthorizationError,
    NotFoundError,
    RateLimitError,
    ServerError,
    TimeoutError,
    ValidationError,
} from '../../errors';

jest.mock('axios');
const mockedAxios = axios as jest.Mocked<typeof axios>;

describe('HttpTransport', () => {
    beforeEach(() => {
        jest.clearAllMocks();
    });

    describe('constructor', () => {
        it('should create an axios instance with default config', () => {
            new HttpTransport({});
            expect(mockedAxios.create).toHaveBeenCalledWith({
                baseURL: 'http://localhost:8000',
                timeout: 30000,
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                },
            });
        });

        it('should create an axios instance with custom config', () => {
            new HttpTransport({
                baseUrl: 'https://example.com',
                apiKey: 'test-key',
                timeout: 5000,
                defaultHeaders: {
                    'X-Custom-Header': 'test',
                },
            });
            expect(mockedAxios.create).toHaveBeenCalledWith({
                baseURL: 'https://example.com',
                timeout: 5000,
                headers: {
                    'Content-Type': 'application/json',
                    'Accept': 'application/json',
                    'Authorization': 'Bearer test-key',
                    'X-Custom-Header': 'test',
                },
            });
        });
    });

    describe('request', () => {
        let transport: HttpTransport;
        let mockAxiosInstance: any;

        beforeEach(() => {
            mockAxiosInstance = {
                request: jest.fn(),
                interceptors: {
                    response: {
                        use: jest.fn(),
                    },
                },
            };
            mockedAxios.create.mockReturnValue(mockAxiosInstance);
            transport = new HttpTransport({});
        });

        it('should call axios.request with correct parameters', async () => {
            mockAxiosInstance.request.mockResolvedValue({ data: { id: 'test-id' } });

            const result = await transport.request({
                method: 'GET',
                path: '/test',
                params: { foo: 'bar' },
                headers: { 'X-Test': 'test' },
            });

            expect(mockAxiosInstance.request).toHaveBeenCalledWith({
                method: 'GET',
                url: '/test',
                params: { foo: 'bar' },
                headers: { 'X-Test': 'test' },
                data: undefined,
            });
            expect(result).toEqual({ id: 'test-id' });
        });

        it('should handle 400 error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                response: {
                    status: 400,
                    data: { message: 'Bad request' },
                },
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(ValidationError);
        });

        it('should handle 401 error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                response: {
                    status: 401,
                    data: { message: 'Unauthorized' },
                },
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(AuthenticationError);
        });

        it('should handle 403 error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                response: {
                    status: 403,
                    data: { message: 'Forbidden' },
                },
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(AuthorizationError);
        });

        it('should handle 404 error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                response: {
                    status: 404,
                    data: { message: 'Not found' },
                },
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(NotFoundError);
        });

        it('should handle 429 error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                response: {
                    status: 429,
                    data: { message: 'Too many requests' },
                },
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(RateLimitError);
        });

        it('should handle 500 error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                response: {
                    status: 500,
                    data: { message: 'Server error' },
                },
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(ServerError);
        });

        it('should handle timeout error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                code: 'ECONNABORTED',
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(TimeoutError);
        });

        it('should handle network error', async () => {
            mockAxiosInstance.request.mockRejectedValue({
                message: 'Network error',
            });

            await expect(
                transport.request({
                    method: 'GET',
                    path: '/test',
                })
            ).rejects.toThrow(ApiError);
        });
    });

    describe('helper methods', () => {
        let transport: HttpTransport;

        beforeEach(() => {
            transport = new HttpTransport({});
            (transport as any).request = jest.fn();
            (transport as any).requestStream = jest.fn();
        });

        it('should call request with GET method', async () => {
            await transport.get('/test', { foo: 'bar' }, { 'X-Test': 'test' });
            expect((transport as any).request).toHaveBeenCalledWith({
                method: 'GET',
                path: '/test',
                params: { foo: 'bar' },
                headers: { 'X-Test': 'test' },
            });
        });

        it('should call request with POST method', async () => {
            await transport.post('/test', { foo: 'bar' }, { 'X-Test': 'test' });
            expect((transport as any).request).toHaveBeenCalledWith({
                method: 'POST',
                path: '/test',
                body: { foo: 'bar' },
                headers: { 'X-Test': 'test' },
            });
        });

        it('should call request with PUT method', async () => {
            await transport.put('/test', { foo: 'bar' }, { 'X-Test': 'test' });
            expect((transport as any).request).toHaveBeenCalledWith({
                method: 'PUT',
                path: '/test',
                body: { foo: 'bar' },
                headers: { 'X-Test': 'test' },
            });
        });

        it('should call request with DELETE method', async () => {
            await transport.delete('/test', { 'X-Test': 'test' });
            expect((transport as any).request).toHaveBeenCalledWith({
                method: 'DELETE',
                path: '/test',
                headers: { 'X-Test': 'test' },
            });
        });

        it('should call requestStream with POST method', async () => {
            await transport.postStream('/test', { foo: 'bar' }, { 'X-Test': 'test' });
            expect((transport as any).requestStream).toHaveBeenCalledWith({
                method: 'POST',
                path: '/test',
                body: { foo: 'bar' },
                headers: { 'X-Test': 'test' },
                stream: true,
            });
        });
    });
});