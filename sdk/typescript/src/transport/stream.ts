/**
 * Convert an AsyncIterable to a ReadableStream
 * @param iterable Async iterable to convert
 * @returns ReadableStream
 */
export function asyncIterableToStream<T>(iterable: AsyncIterable<T>): ReadableStream<T> {
    return new ReadableStream({
        async start(controller) {
            try {
                for await (const chunk of iterable) {
                    controller.enqueue(chunk);
                }
                controller.close();
            } catch (error) {
                controller.error(error);
            }
        },
    });
}

/**
 * Convert a ReadableStream to an AsyncIterable
 * @param stream ReadableStream to convert
 * @returns AsyncIterable
 */
export async function* streamToAsyncIterable<T>(stream: ReadableStream<T>): AsyncIterable<T> {
    const reader = stream.getReader();

    try {
        while (true) {
            const { done, value } = await reader.read();

            if (done) {
                break;
            }

            yield value;
        }
    } finally {
        reader.releaseLock();
    }
}