//! Main library entry point for openapi_client implementation.

#![allow(unused_imports)]

use async_trait::async_trait;
use futures::{future, Stream, StreamExt, TryFutureExt, TryStreamExt};
use hyper::server::conn::Http;
use hyper::service::Service;
use log::{error, info};
use s3::bucket::Bucket;
use s3::creds::Credentials;
use std::env;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use swagger::auth::MakeAllowAllAuthenticator;
use swagger::EmptyContext;
use swagger::{Has, XSpanIdString};
use tokio::net::TcpListener;

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
use openssl::ssl::{Ssl, SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use openapi_client::models;

/// Builds an SSL implementation for Simple HTTPS from some hard-coded file names
pub async fn create(addr: &str, https: bool) {
    let addr = addr.parse().expect("Failed to parse bind address");

    let server = Server::new();

    let service = MakeService::new(server);

    let service = MakeAllowAllAuthenticator::new(service, "cosmo");

    #[allow(unused_mut)]
    let mut service =
        openapi_client::server::context::MakeAddContext::<_, EmptyContext>::new(service);

    if https {
        #[cfg(any(target_os = "macos", target_os = "windows", target_os = "ios"))]
        {
            unimplemented!("SSL is not implemented for the examples on MacOS, Windows or iOS");
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "ios")))]
        {
            let mut ssl = SslAcceptor::mozilla_intermediate_v5(SslMethod::tls())
                .expect("Failed to create SSL Acceptor");

            // Server authentication
            ssl.set_private_key_file("examples/server-key.pem", SslFiletype::PEM)
                .expect("Failed to set private key");
            ssl.set_certificate_chain_file("examples/server-chain.pem")
                .expect("Failed to set certificate chain");
            ssl.check_private_key()
                .expect("Failed to check private key");

            let tls_acceptor = ssl.build();
            let tcp_listener = TcpListener::bind(&addr).await.unwrap();

            loop {
                if let Ok((tcp, _)) = tcp_listener.accept().await {
                    let ssl = Ssl::new(tls_acceptor.context()).unwrap();
                    let addr = tcp.peer_addr().expect("Unable to get remote address");
                    let service = service.call(addr);

                    tokio::spawn(async move {
                        let tls = tokio_openssl::SslStream::new(ssl, tcp).map_err(|_| ())?;
                        let service = service.await.map_err(|_| ())?;

                        Http::new()
                            .serve_connection(tls, service)
                            .await
                            .map_err(|_| ())
                    });
                }
            }
        }
    } else {
        // Using HTTP
        hyper::server::Server::bind(&addr)
            .serve(service)
            .await
            .unwrap()
    }
}

#[derive(Copy, Clone)]
pub struct Server<C> {
    marker: PhantomData<C>,
}

impl<C> Server<C> {
    pub fn new() -> Self {
        Server {
            marker: PhantomData,
        }
    }
}

use openapi_client::server::MakeService;
use openapi_client::{Api, FilesGetResponse};
use std::error::Error;
use swagger::ApiError;

#[async_trait]
impl<C> Api<C> for Server<C>
where
    C: Has<XSpanIdString> + Send + Sync,
{
    async fn files_get(&self, context: &C) -> Result<FilesGetResponse, ApiError> {
        let _context = context.clone();

        let bucket_name = match env::var("AWS_S3_BUCKET") {
            Ok(bucket_name) => bucket_name,
            Err(err) => return Err(ApiError(format!("Invalid bucket: {}", err))),
        };

        let region = match env::var("AWS_REGION") {
            Ok(region) => region,
            Err(err) => {
                return Err(ApiError(format!(
                    "Missing AWS_REGION env variable: {}",
                    err
                )))
            }
        };

        let region = match region.parse() {
            Ok(region) => region,
            Err(err) => return Err(ApiError(format!("Invalid region: {}", err))),
        };

        let credentials = match Credentials::from_env() {
            Ok(credentials) => credentials,
            Err(err) => return Err(ApiError(format!("Invalid credentials: {}", err))),
        };

        let bucket = match Bucket::new(&bucket_name, region, credentials) {
            Ok(bucket) => bucket,
            Err(err) => return Err(ApiError(format!("Invalid bucket: {}", err))),
        };

        let results = match bucket.list(String::default(), Some("/".to_string())).await {
            Ok(results) => results,
            Err(err) => {
                return Err(ApiError(format!("Failed to list bucket: {}", err)));
            }
        };

        let files = results
            .first()
            .unwrap()
            .contents
            .iter()
            .map(|x| models::FilesGet200ResponseInner {
                file_name: Some(x.key.to_owned()),
            })
            .collect();

        Ok(FilesGetResponse::AListOfItemsFromTheObjectBucket(files))
    }
}
