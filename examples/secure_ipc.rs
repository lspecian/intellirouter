use std::path::PathBuf;
use std::sync::Arc;

use intellirouter::modules::ipc::security::{
    JwtAuthenticator, JwtConfig, RoleConfig, SecureGrpcClientBuilder, SecureGrpcServerBuilder,
    SecureRedisClientBuilder, TlsConfig,
};
use tokio::net::TcpListener;
use tonic::{Request, Response, Status};

// Example gRPC service definition
#[derive(Debug)]
pub struct ExampleService;

#[tonic::async_trait]
impl example_service_server::ExampleService for ExampleService {
    async fn get_example(
        &self,
        request: Request<example_service::ExampleRequest>,
    ) -> Result<Response<example_service::ExampleResponse>, Status> {
        println!("Got a request: {:?}", request);

        let response = example_service::ExampleResponse {
            message: format!("Hello, {}!", request.into_inner().name),
        };

        Ok(Response::new(response))
    }
}

pub mod example_service {
    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ExampleRequest {
        #[prost(string, tag = "1")]
        pub name: String,
    }

    #[derive(Clone, PartialEq, ::prost::Message)]
    pub struct ExampleResponse {
        #[prost(string, tag = "1")]
        pub message: String,
    }
}

pub mod example_service_server {
    use super::example_service::*;
    use tonic::{Request, Response, Status};

    #[tonic::async_trait]
    pub trait ExampleService: Send + Sync + 'static {
        async fn get_example(
            &self,
            request: Request<ExampleRequest>,
        ) -> Result<Response<ExampleResponse>, Status>;
    }

    #[derive(Debug)]
    pub struct ExampleServiceServer<T>(pub T);

    impl<T: ExampleService> ExampleServiceServer<T> {
        pub fn new(inner: T) -> Self {
            Self(inner)
        }
    }

    impl<T: ExampleService> tonic::server::NamedService for ExampleServiceServer<T> {
        const NAME: &'static str = "example_service";
    }

    #[tonic::async_trait]
    impl<T: ExampleService> tonic::server::Service<tonic::body::BoxBody> for ExampleServiceServer<T> {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
        >;

        fn poll_ready(
            &mut self,
            _cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), Self::Error>> {
            std::task::Poll::Ready(Ok(()))
        }

        fn call(&mut self, req: tonic::Request<tonic::body::BoxBody>) -> Self::Future {
            let inner = self.0.clone();

            Box::pin(async move {
                let method = req.uri().path();
                match method {
                    "/example_service/GetExample" => {
                        let codec = tonic::codec::ProstCodec::default();
                        let path =
                            http::uri::PathAndQuery::from_static("/example_service/GetExample");

                        let mut req = req.into_request();
                        req.extensions_mut()
                            .insert(GrpcMethod::new("example_service", "GetExample"));

                        let request = codec.decode(req).unwrap();
                        let response = inner.get_example(request).await?;
                        let mut resp = Response::new(codec.encode(response).unwrap());
                        resp.extensions_mut()
                            .insert(GrpcMethod::new("example_service", "GetExample"));
                        Ok(resp.map(|body| tonic::body::BoxBody::new(body)))
                    }
                    _ => Err(Status::unimplemented(format!(
                        "Method {} not implemented",
                        method
                    ))),
                }
            })
        }
    }

    #[derive(Debug)]
    struct GrpcMethod<'a> {
        service: &'a str,
        method: &'a str,
    }

    impl<'a> GrpcMethod<'a> {
        fn new(service: &'a str, method: &'a str) -> Self {
            Self { service, method }
        }
    }
}

pub mod example_service_client {
    use super::example_service::*;
    use tonic::codegen::InterceptedService;
    use tonic::transport::Channel;

    #[derive(Debug, Clone)]
    pub struct ExampleServiceClient<T> {
        inner: T,
    }

    impl ExampleServiceClient<tonic::transport::Channel> {
        pub fn new(channel: Channel) -> Self {
            Self { inner: channel }
        }

        pub fn with_interceptor<F>(
            channel: Channel,
            interceptor: F,
        ) -> ExampleServiceClient<InterceptedService<Channel, F>>
        where
            F: tonic::service::Interceptor + Send + Sync + 'static,
        {
            ExampleServiceClient::new(InterceptedService::new(channel, interceptor))
        }
    }

    impl<T> ExampleServiceClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<tonic::Status>,
        T::ResponseBody: tonic::codegen::Body<Data = tonic::codegen::Bytes> + Send + 'static,
        <T::ResponseBody as tonic::codegen::Body>::Error: Into<tonic::Status> + Send,
    {
        pub async fn get_example(
            &mut self,
            request: impl tonic::IntoRequest<ExampleRequest>,
        ) -> Result<tonic::Response<ExampleResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/example_service/GetExample");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }

    impl From<tonic::transport::Channel> for ExampleServiceClient<tonic::transport::Channel> {
        fn from(channel: tonic::transport::Channel) -> Self {
            Self::new(channel)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create JWT configuration
    let jwt_config = JwtConfig {
        secret: "your-secret-key".to_string(),
        issuer: "intellirouter".to_string(),
        audience: "intellirouter-services".to_string(),
        expiration_seconds: 3600, // 1 hour
    };

    let jwt_authenticator = Arc::new(JwtAuthenticator::new(jwt_config));

    // Create role configuration
    let mut role_config = RoleConfig::new();
    role_config.add_role("example_service", "read_example");
    role_config.add_role("example_client", "request_example");

    // Create TLS configuration (commented out as it requires actual certificates)
    // let tls_config = TlsConfig::new(
    //     PathBuf::from("path/to/cert.crt"),
    //     PathBuf::from("path/to/key.key"),
    //     PathBuf::from("path/to/ca.crt"),
    // );

    // Start the server in a separate task
    let server_jwt_authenticator = jwt_authenticator.clone();
    let server_handle = tokio::spawn(async move {
        // Create the service
        let service = ExampleService;
        let service = example_service_server::ExampleServiceServer::new(service);

        // Create the secure server builder
        let server_builder = SecureGrpcServerBuilder::new()
            // .with_tls(tls_config.clone()) // Uncomment when you have actual certificates
            .with_jwt(
                server_jwt_authenticator.clone(),
                vec!["request_example".to_string()],
            );

        // Build the server
        let router = server_builder
            .build::<example_service_server::ExampleServiceServer<_>>()
            .unwrap();
        let router = server_builder.add_service(router, service);

        // Bind to an address
        let addr = "[::1]:50051".parse().unwrap();
        println!("Server listening on {}", addr);

        // Start the server
        router.serve(addr).await.unwrap();
    });

    // Wait a moment for the server to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create a secure client
    let client = SecureGrpcClientBuilder::<example_service_client::ExampleServiceClient<_>>::new(
        "localhost",
    )
    // .with_tls(tls_config.clone()) // Uncomment when you have actual certificates
    .with_jwt(
        jwt_authenticator.clone(),
        "example_client".to_string(),
        vec!["request_example".to_string()],
    )
    .build("http://[::1]:50051")
    .await?;

    // Create a request
    let request = example_service::ExampleRequest {
        name: "Secure World".to_string(),
    };

    // Send the request
    let mut client = client;
    let response = client.get_example(request).await?;
    println!("Response: {:?}", response);

    // Create a secure Redis client (commented out as it requires a running Redis instance)
    // let redis_client = SecureRedisClientBuilder::new()
    //     // .with_tls(tls_config.clone()) // Uncomment when you have actual certificates
    //     .with_jwt(
    //         jwt_authenticator.clone(),
    //         "example_client".to_string(),
    //         vec!["request_example".to_string()],
    //     )
    //     .build("redis://localhost:6379")
    //     .await?;
    //
    // // Publish a message
    // redis_client.publish("example_channel", b"Hello, secure world!").await?;
    //
    // // Subscribe to a channel
    // let subscription = redis_client.subscribe("example_channel").await?;
    // if let Some(message) = subscription.next_message().await? {
    //     println!("Received message: {:?}", message);
    // }

    // Wait for the server to finish (which it won't in this example)
    // In a real application, you would have a proper shutdown mechanism
    server_handle.await?;

    Ok(())
}
