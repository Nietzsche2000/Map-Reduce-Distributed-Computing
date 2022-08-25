//! A MapReduce worker.
//!

//! CODE WRITTEN BY MONISHWARAN MAHESWARAN SUMMER 2022

use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;
use std::thread;
use std::time::Duration;

use crate::rpc::coordinator::coordinator_client::CoordinatorClient;
use crate::rpc::worker::*;
use crate::*;
use tonic::transport::{Channel, Server};

use crate::rpc::worker::*;

use anyhow::{Result};
use crate::rpc::coordinator::{ConnectErrorRequest, HeartBeatRequest, JobDoneRequest, JobFailureRequest, JobRequestReply, JobRequestRequest, RegisterRequest};

pub mod args;


use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::app::named;
use std::error::Error;
use std::fs::read;
use std::os::unix::raw::off_t;
use std::sync::{Arc};
use bytes::BufMut;
use tokio::sync::Mutex;
use rand::distributions::Open01;
use serde::de::Unexpected::Bytes;
use tokio::sync::RwLock;
use tonic::{Code, Request, Response, Status};
use tonic::codegen::Body;
use crate::coordinator::Coordinator;

#[derive(Clone)]
pub struct Worker {
    id: u32,
    jobs: Arc<Mutex<HashMap<u32, HashMap<u32, Vec<bytes::Bytes>>>>>,
}

async fn do_map_job(wx: &mut Worker, inner: JobRequestReply) -> Option<String> {
    let input_file = inner.file;
    let parent_job_id = inner.parent_job_id;
    let reduce_buckets = inner.n_reduce;
    let aux = inner.args;
    let mut content = Vec::new();
    let mut file = tokio::fs::File::open(&input_file).await;
    let mut jobs = wx.jobs.lock().await;

    /** GET THE BUCKETS CORRESPONDING TO A JOB */
    let buckets = jobs.entry(parent_job_id).or_insert(HashMap::new());

    match file {
        Ok(mut f) => {
            let error_read = f.read_to_end(&mut content).await;
            match error_read {
                Ok(_) => {
                    let content = bytes::Bytes::from(content);
                    let aux_b = bytes::Bytes::from(aux);
                    let kv = KeyValue {
                        key: bytes::Bytes::from(input_file),
                        value: content,
                    };
                    let application = named(inner.app.as_str()).unwrap();
                    let out_r = (application.map_fn)(kv, aux_b);
                    match out_r {
                        Ok(out) => {
                            for rkv_re in out {
                                match rkv_re {
                                    Ok(kv) => {
                                        let key = kv.clone().key;
                                        let h = (ihash(key.as_ref()) % reduce_buckets) + 1;
                                        let mut bucket = buckets.entry(h).or_insert(Vec::new());
                                        bucket.push(kv.key.clone());
                                        bucket.push(kv.value.clone());
                                    }
                                    Err(e) => return Some(format!("{:#}", e)),
                                }
                            }
                        }
                        Err(e) => return Some(format!("{:#}", e)),
                    }
                }
                Err(e) => return Some(format!("{:#}", e)),
            }
            return None;
        }
        Err(e) => return Some(format!("{:#}", e)),
    };
}

async fn send_heart_beat(id: u32) -> Result<()> {
    let mut client = CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR)).await?;
    tokio::spawn(async move {
        // Process each socket concurrently.
        loop {
            let res = client.heart_beat(HeartBeatRequest { worker_id: id }).await;
            tokio::time::sleep(Duration::from_millis(HEARTBEAT_INTERVAL_MS)).await;
        }
    });
    return Ok(());
}

async fn send_map_job_done(id: u32, inner: &JobRequestReply) -> Result<()> {
    let mut client = CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR)).await?;
    println!("WORKER:{} DONE WITH JOB {}", id, inner.job_id.clone());
    client.job_done(JobDoneRequest {
        worker_id: id,
        job_id: inner.parent_job_id.clone(),
        sub_job_id: inner.job_id.clone(),
        job_type: inner.job_type,
    }).await?;
    return Ok(());
}

async fn send_reduce_job_done(id: u32, inner: &JobRequestReply) -> Result<()> {
    let mut client = CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR)).await?;
    println!("WORKER:{} DONE WITH REDUCE JOB {}", id, inner.job_id.clone());
    client.job_done(JobDoneRequest {
        worker_id: id,
        job_id: inner.parent_job_id.clone(),
        sub_job_id: inner.job_id.clone(),
        job_type: inner.job_type,
    }).await?;
    return Ok(());
}

async fn send_connect_error(client: &mut CoordinatorClient<Channel>, id: u32, worker_id: u32, parent_job_id: u32, sub_job_id: u32) -> Result<()> {
    println!("WORKER:{} SENDING CONNECT ERROR {}", id, worker_id);
    client.connect_error(ConnectErrorRequest {
        id: id,
        worker_id: worker_id,
        parent_job_id: parent_job_id,
        sub_job_id: sub_job_id,
    }).await?;
    return Ok(());
}

async fn send_job_error(client: &mut CoordinatorClient<Channel>, parent_job_id: u32, error: String) -> Result<()> {
    println!("WORKER SENDING JOB ERROR {}", parent_job_id);
    client.job_failure(JobFailureRequest {
        parent_job_id: parent_job_id,
        error: error,
    }).await?;
    return Ok(());
}

impl Worker {
    pub async fn new() -> Result<Self> {
        // Connect to the coordinator
        let mut client = CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR)).await?;
        let res = client.register(RegisterRequest {}).await?;
        let id = res.into_inner().id;
        Ok(Worker { id, jobs: Arc::new(Mutex::new(HashMap::new())) })
    }

    pub async fn run(mut self) -> Result<()> {
        /* SEND HEART BEAT */
        send_heart_beat(self.id.clone()).await?;

        /** CONNECT TO COORDINATOR */
        let mut client = CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR)).await?;
        loop {
            /** GET A JOB */
            println!("WORKER: {} REQ JOB.", self.id.clone());
            let res = client.job_request(JobRequestRequest { worker_id: self.id.clone() }).await?;
            let inner = res.into_inner();

            /* IF A MAP JOB */
            if inner.valid == 1 && inner.job_type == 0 {
                /** DO THE MAP JOB */
                let good_map = do_map_job(&mut self, inner.clone()).await;
                if good_map != None {
                    send_job_error(&mut client, inner.parent_job_id, good_map.unwrap().to_string()).await.expect(" ");
                    continue;
                }
                /** LET THE COORDINATOR KNOW ABOUT COMPLETION */
                send_map_job_done(self.id.clone(), &inner).await?;
            } else if inner.valid == 1 && inner.job_type == 1 {
                /** IF A REDUCE JOB */
                let workers: HashSet<u32> = inner.worker_ids.iter().cloned().collect();
                // let workers = inner.worker_ids.clone();
                println!("WORKER IDS: {:?}", workers);
                let mut kv_g = HashMap::new();
                println!("STARTING REDUCE BY WORKER {}", self.id);
                /** FOR EVERY WORKER */
                let mut c_error = 0;
                for w in workers {
                    let mut worker_connect = connect(w).await;

                    match worker_connect {
                        Ok(mut worker) => {
                            /** REQUEST WORKER TO SEND MAP DATA */
                            let rep_connect = worker.map_data_request(MapDataRequestRequest {
                                job_id: inner.parent_job_id,
                                bucket_num: inner.job_id,
                                args: inner.args.clone(),
                                app: inner.app.clone(),
                                output_dir: inner.output_dir.clone(),
                            }).await;

                            match rep_connect {
                                Ok(rep) => {
                                    let bucket = rep.get_ref().data.clone();
                                    let data = bytes::Bytes::from(bucket);
                                    let mut reader = codec::LengthDelimitedReader::new(data);
                                    loop {
                                        let key = reader.next();
                                        let val = reader.next();
                                        if key == None && val == None {
                                            break;
                                        }
                                        assert!(val != None);
                                        let k_unwrapped = key.unwrap();
                                        let v_unwrapped = val.unwrap();
                                        let l = kv_g.entry(k_unwrapped.clone()).or_insert(Vec::new());
                                        l.push(v_unwrapped.clone());
                                    }
                                }
                                Err(_) => {
                                    send_connect_error(&mut client, self.id.clone(), w.clone(), inner.parent_job_id.clone(), inner.job_id.clone()).await.expect("TODO: panic message");
                                    c_error = 1;
                                    break;
                                }
                            }
                        }
                        Err(_) => {
                            send_connect_error(&mut client, self.id.clone(), w.clone(), inner.parent_job_id.clone(), inner.job_id.clone()).await.expect("TODO: panic message");
                            c_error = 1;
                            break;
                        }
                    }
                }

                let mut j_error = 0;
                if c_error != 1 {
                    let application = named(inner.app.as_str()).unwrap();
                    let mut writer_r = codec::LengthDelimitedWriter::new();
                    let args = inner.args.clone();
                    let aux = bytes::Bytes::from(args.clone());
                    for (key, values) in kv_g {
                        let output_r = (application.reduce_fn)(key.clone(), Box::new(values.into_iter()), aux.clone());
                        match output_r {
                            Ok(output) => {
                                writer_r.send(key.clone());
                                writer_r.send(output);
                            }
                            Err(e) => {
                                send_job_error(&mut client, inner.parent_job_id, format!("{:#}", e)).await?;
                                j_error = 1;
                                break;
                            }
                        }
                    }

                    if j_error == 1 {
                        break;
                    }

                    let buf = writer_r.finish().freeze();

                    let name = format!("{}/mr-out-{}", inner.output_dir, inner.job_id - 1);
                    let mut out_file = tokio::fs::File::create(name).await?;
                    out_file.write_all(&buf).await?;

                    /** LET THE COORDINATOR KNOW ABOUT COMPLETION */
                    send_reduce_job_done(self.id.clone(), &inner).await?;
                }
            } else {
                tokio::time::sleep(Duration::from_millis(WAIT_TIME_MS)).await;
            }
        }
        return Ok(());
    }
}


#[tonic::async_trait]
impl worker_server::Worker for Worker {
    async fn map_data_request(&self, request: Request<MapDataRequestRequest>) -> std::result::Result<Response<MapDataRequestReply>, Status> {
        let inner = request.get_ref();
        let job_id = inner.job_id;
        let bucket_num = inner.bucket_num;
        let mut jobs = self.jobs.lock().await;
        let buckets = jobs.get_mut(&job_id).unwrap();
        let bucket = buckets.get(&bucket_num);
        if bucket == None {
            return Ok(Response::new(MapDataRequestReply { data: bytes::Bytes::new().to_vec() }));
        }
        let mut writer = codec::LengthDelimitedWriter::new();

        for b in bucket.unwrap() {
            writer.send(b.clone());
        }

        let buf = writer.finish().freeze();
        return Ok(Response::new(MapDataRequestReply { data: buf.to_vec() }));
    }
}

async fn worker_server(worker: Worker) -> Result<()> {
    let addr = get_addr(worker.id);
    let svc = worker_server::WorkerServer::new(worker);
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

pub async fn start(_args: args::Args) -> Result<()> {
    let worker: Worker = Worker::new().await.unwrap();
    let server = worker.clone();
    tokio::spawn(async move { worker_server(server).await });
    worker.run().await?;
    Ok(())
}

fn get_port(id: WorkerId) -> u16 {
    let port = INITIAL_WORKER_PORT as WorkerId + id;
    assert!(port <= u16::MAX as WorkerId);
    port as u16
}

fn get_addr(id: WorkerId) -> SocketAddr {
    format!("127.0.0.1:{}", get_port(id)).parse().unwrap()
}

async fn connect(id: WorkerId) -> Result<worker_client::WorkerClient<Channel>> {
    let client =
        worker_client::WorkerClient::connect(format!("http://127.0.0.1:{}", get_port(id))).await?;
    Ok(client)
}
