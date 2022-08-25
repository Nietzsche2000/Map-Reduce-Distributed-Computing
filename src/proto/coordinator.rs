#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartBeatRequest {
    #[prost(uint32, tag="1")]
    pub worker_id: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HeartBeatReply {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterRequest {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RegisterReply {
    #[prost(uint32, tag="1")]
    pub id: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubmitJobRequest {
    #[prost(string, repeated, tag="1")]
    pub files: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(string, tag="2")]
    pub output_dir: ::prost::alloc::string::String,
    #[prost(string, tag="3")]
    pub app: ::prost::alloc::string::String,
    #[prost(uint32, tag="4")]
    pub n_reduce: u32,
    #[prost(bytes="vec", tag="5")]
    pub args: ::prost::alloc::vec::Vec<u8>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SubmitJobReply {
    #[prost(uint32, tag="1")]
    pub job_id: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PollJobRequest {
    #[prost(uint32, tag="1")]
    pub job_id: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PollJobReply {
    #[prost(bool, tag="1")]
    pub done: bool,
    #[prost(bool, tag="2")]
    pub failed: bool,
    #[prost(string, repeated, tag="3")]
    pub errors: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobRequestRequest {
    #[prost(uint32, tag="1")]
    pub worker_id: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobRequestReply {
    #[prost(string, tag="1")]
    pub file: ::prost::alloc::string::String,
    #[prost(bytes="vec", tag="2")]
    pub args: ::prost::alloc::vec::Vec<u8>,
    #[prost(uint32, tag="3")]
    pub job_type: u32,
    #[prost(string, tag="4")]
    pub app: ::prost::alloc::string::String,
    #[prost(uint32, repeated, tag="5")]
    pub worker_ids: ::prost::alloc::vec::Vec<u32>,
    #[prost(uint32, tag="6")]
    pub parent_job_id: u32,
    #[prost(uint32, tag="7")]
    pub job_id: u32,
    #[prost(uint32, tag="8")]
    pub reduce_bucket: u32,
    #[prost(uint32, tag="9")]
    pub valid: u32,
    #[prost(uint32, tag="10")]
    pub n_reduce: u32,
    ///  uint32 map_job_id = 12;
    #[prost(string, tag="11")]
    pub output_dir: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobDoneRequest {
    #[prost(uint32, tag="1")]
    pub worker_id: u32,
    #[prost(uint32, tag="2")]
    pub job_id: u32,
    #[prost(uint32, tag="3")]
    pub sub_job_id: u32,
    #[prost(uint32, tag="4")]
    pub job_type: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobDoneReply {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectErrorRequest {
    #[prost(uint32, tag="1")]
    pub id: u32,
    #[prost(uint32, tag="2")]
    pub worker_id: u32,
    #[prost(uint32, tag="3")]
    pub parent_job_id: u32,
    #[prost(uint32, tag="4")]
    pub sub_job_id: u32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectErrorReply {
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobFailureRequest {
    #[prost(uint32, tag="1")]
    pub parent_job_id: u32,
    #[prost(string, tag="2")]
    pub error: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JobFailureReply {
}
/// Generated client implementations.
pub mod coordinator_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct CoordinatorClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl CoordinatorClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> CoordinatorClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> CoordinatorClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            CoordinatorClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with `gzip`.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        /// Enable decompressing responses with `gzip`.
        #[must_use]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn submit_job(
            &mut self,
            request: impl tonic::IntoRequest<super::SubmitJobRequest>,
        ) -> Result<tonic::Response<super::SubmitJobReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/SubmitJob",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn poll_job(
            &mut self,
            request: impl tonic::IntoRequest<super::PollJobRequest>,
        ) -> Result<tonic::Response<super::PollJobReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/PollJob",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn register(
            &mut self,
            request: impl tonic::IntoRequest<super::RegisterRequest>,
        ) -> Result<tonic::Response<super::RegisterReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/Register",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn heart_beat(
            &mut self,
            request: impl tonic::IntoRequest<super::HeartBeatRequest>,
        ) -> Result<tonic::Response<super::HeartBeatReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/HeartBeat",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn job_request(
            &mut self,
            request: impl tonic::IntoRequest<super::JobRequestRequest>,
        ) -> Result<tonic::Response<super::JobRequestReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/JobRequest",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn job_done(
            &mut self,
            request: impl tonic::IntoRequest<super::JobDoneRequest>,
        ) -> Result<tonic::Response<super::JobDoneReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/JobDone",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn connect_error(
            &mut self,
            request: impl tonic::IntoRequest<super::ConnectErrorRequest>,
        ) -> Result<tonic::Response<super::ConnectErrorReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/ConnectError",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn job_failure(
            &mut self,
            request: impl tonic::IntoRequest<super::JobFailureRequest>,
        ) -> Result<tonic::Response<super::JobFailureReply>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/coordinator.Coordinator/JobFailure",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod coordinator_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    ///Generated trait containing gRPC methods that should be implemented for use with CoordinatorServer.
    #[async_trait]
    pub trait Coordinator: Send + Sync + 'static {
        async fn submit_job(
            &self,
            request: tonic::Request<super::SubmitJobRequest>,
        ) -> Result<tonic::Response<super::SubmitJobReply>, tonic::Status>;
        async fn poll_job(
            &self,
            request: tonic::Request<super::PollJobRequest>,
        ) -> Result<tonic::Response<super::PollJobReply>, tonic::Status>;
        async fn register(
            &self,
            request: tonic::Request<super::RegisterRequest>,
        ) -> Result<tonic::Response<super::RegisterReply>, tonic::Status>;
        async fn heart_beat(
            &self,
            request: tonic::Request<super::HeartBeatRequest>,
        ) -> Result<tonic::Response<super::HeartBeatReply>, tonic::Status>;
        async fn job_request(
            &self,
            request: tonic::Request<super::JobRequestRequest>,
        ) -> Result<tonic::Response<super::JobRequestReply>, tonic::Status>;
        async fn job_done(
            &self,
            request: tonic::Request<super::JobDoneRequest>,
        ) -> Result<tonic::Response<super::JobDoneReply>, tonic::Status>;
        async fn connect_error(
            &self,
            request: tonic::Request<super::ConnectErrorRequest>,
        ) -> Result<tonic::Response<super::ConnectErrorReply>, tonic::Status>;
        async fn job_failure(
            &self,
            request: tonic::Request<super::JobFailureRequest>,
        ) -> Result<tonic::Response<super::JobFailureReply>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct CoordinatorServer<T: Coordinator> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Coordinator> CoordinatorServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for CoordinatorServer<T>
    where
        T: Coordinator,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/coordinator.Coordinator/SubmitJob" => {
                    #[allow(non_camel_case_types)]
                    struct SubmitJobSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::SubmitJobRequest>
                    for SubmitJobSvc<T> {
                        type Response = super::SubmitJobReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SubmitJobRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).submit_job(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = SubmitJobSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/coordinator.Coordinator/PollJob" => {
                    #[allow(non_camel_case_types)]
                    struct PollJobSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::PollJobRequest>
                    for PollJobSvc<T> {
                        type Response = super::PollJobReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::PollJobRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).poll_job(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PollJobSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/coordinator.Coordinator/Register" => {
                    #[allow(non_camel_case_types)]
                    struct RegisterSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::RegisterRequest>
                    for RegisterSvc<T> {
                        type Response = super::RegisterReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RegisterRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).register(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = RegisterSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/coordinator.Coordinator/HeartBeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartBeatSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::HeartBeatRequest>
                    for HeartBeatSvc<T> {
                        type Response = super::HeartBeatReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::HeartBeatRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).heart_beat(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HeartBeatSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/coordinator.Coordinator/JobRequest" => {
                    #[allow(non_camel_case_types)]
                    struct JobRequestSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::JobRequestRequest>
                    for JobRequestSvc<T> {
                        type Response = super::JobRequestReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::JobRequestRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).job_request(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = JobRequestSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/coordinator.Coordinator/JobDone" => {
                    #[allow(non_camel_case_types)]
                    struct JobDoneSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::JobDoneRequest>
                    for JobDoneSvc<T> {
                        type Response = super::JobDoneReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::JobDoneRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).job_done(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = JobDoneSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/coordinator.Coordinator/ConnectError" => {
                    #[allow(non_camel_case_types)]
                    struct ConnectErrorSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::ConnectErrorRequest>
                    for ConnectErrorSvc<T> {
                        type Response = super::ConnectErrorReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ConnectErrorRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move {
                                (*inner).connect_error(request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ConnectErrorSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/coordinator.Coordinator/JobFailure" => {
                    #[allow(non_camel_case_types)]
                    struct JobFailureSvc<T: Coordinator>(pub Arc<T>);
                    impl<
                        T: Coordinator,
                    > tonic::server::UnaryService<super::JobFailureRequest>
                    for JobFailureSvc<T> {
                        type Response = super::JobFailureReply;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::JobFailureRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).job_failure(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = JobFailureSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Coordinator> Clone for CoordinatorServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Coordinator> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Coordinator> tonic::transport::NamedService for CoordinatorServer<T> {
        const NAME: &'static str = "coordinator.Coordinator";
    }
}
