//! The MapReduce coordinator.
//!

//! CODE WRITTEN BY MONISHWARAN MAHESWARAN SUMMER 2022

use std::any::Any;
use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use std::sync::{Arc};
use anyhow::Result;
use clap::builder::TypedValueParser;
use futures_util::future::err;
use futures_util::SinkExt;
use itertools::Itertools;
use serde::de::Unexpected::Map;
use tokio::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use tonic::transport::Server;
use tonic::{Code, Request, Response, Status};

use crate::rpc::coordinator::*;
use crate::*;
use crate::app::named;
use crate::coordinator::JobType::{MapJob, ReduceJob};

pub mod args;

#[derive(PartialEq, Clone)]
pub enum JobType {
    MapJob,
    ReduceJob,
}

#[derive(PartialEq, Clone)]
pub struct SubJob {
    /** SUB JOB ID */
    id: u32,
    /** ID OF PARENT JOB */
    parent_job_id: u32,
    /** INPUT FILE FOR MAP */
    file: String,
    /** WHO WORKED ON THIS JOB */
    worker_id: u32,
    /** WHETHER MAP OR REDUCE TYPE JOB */
    type_job: JobType,
    /** WHAT IS THE MAP APP */
    app: String,
    /** NUM OF REDUCE BUCKETS */
    n_reduce: u32,
    /** INPUT ARGS FOR MAP */
    args: Vec<u8>,
    /** STATE WHETHER JOB COMPLETED */
    done: bool,
    /** STATE WHETHER JOB FAILED */
    failed: bool,
    /** OUTPUT DIR */
    output_dir: String,
}

pub struct Job {
    /** JOB ID */
    job_id: u32,
    /** MAP JOB ID AND THE MAP JOBS */
    map_jobs: HashMap<u32, SubJob>,
    /** REDUCE JOB ID AND THE REDUCE JOB */
    reduce_jobs: HashMap<u32, SubJob>,
    done: bool,
    failed: bool,
    error_arr: Vec<String>,
}

impl Job {
    fn new(j_id: u32) -> Self {
        Job {
            job_id: j_id,
            map_jobs: HashMap::new(),
            reduce_jobs: HashMap::new(),
            done: false,
            failed: false,
            error_arr: Vec::new(),
        }
    }

    /** ADD A JOB */
    fn add_job(&mut self, j: SubJob, t: JobType) {
        if t == MapJob {
            self.map_jobs.insert(j.id, j);
        } else {
            self.reduce_jobs.insert(j.id, j);
        }
    }

    /** RETURNS WHETHER ALL MAP JOBS OF A JOB ARE DONE */
    fn all_map_done(&self) -> bool {
        for j in &self.map_jobs {
            let s = j.1;
            if !s.done {
                return false;
            }
        }
        return true;
    }

    /** CHECK WHETHER ALL REDUCE JOBS ARE DONE */
    fn all_reduce_done(&self) -> bool {
        for j in &self.reduce_jobs {
            let s = j.1;
            if !s.done {
                return false;
            }
        }
        return true;
    }


    /** MARK A MAP JOB AS DONE */
    fn map_job_done(&mut self, sub_job_id: u32) {
        let sub_job = self.map_jobs.get_mut(&sub_job_id);
        match sub_job {
            Some(sj) => {
                sj.done = true;
            }
            None => {}
        }
    }

    /** MARK A REDUCE JOB AS DONE */
    fn reduce_job_done(&mut self, sub_job_id: u32) {
        let sub_job = self.reduce_jobs.get_mut(&sub_job_id);
        match sub_job {
            Some(sj) => {
                sj.done = true;
            }
            None => {}
        }
    }

    /** RETURN IF ALL JOBS ARE DONE */
    fn all_done(&mut self) -> bool {
        self.done = self.all_map_done() && self.all_reduce_done();
        return self.done;
    }

    /** SET THE STATUS OF DONE */
    fn set_done(&mut self, s: bool) {
        self.done = s;
    }

    /** SET THE STATUS OF FAILED */
    fn set_failed(&mut self, s: bool) {
        self.failed = s;
    }

    /** RETURN A JOB REQUEST OF NEXT AVAILABLE MAP JOB */
    fn assign_and_ret_map_job(&mut self, worker_id: u32) -> Option<(JobRequestReply, u32, SubJob)> { // todo:
        for sj in self.map_jobs.iter_mut() {
            let sub_job = sj.1;
            if !sub_job.done && sub_job.worker_id == 0 {
                let job_req_req = JobRequestReply {
                    file: sub_job.file.to_string(),
                    args: sub_job.args.clone(),
                    // 0 is map job
                    job_type: 0,
                    app: sub_job.app.to_string(),
                    worker_ids: vec![],
                    parent_job_id: sub_job.parent_job_id,
                    job_id: sub_job.id,
                    reduce_bucket: 0,
                    valid: 1,
                    n_reduce: sub_job.n_reduce,
                    output_dir: sub_job.output_dir.clone(),
                };
                sub_job.worker_id = worker_id;
                println!("SENDING MAP JOB {} TO WORKER {}", sub_job.id, worker_id);
                return Some((job_req_req, sub_job.parent_job_id, sub_job.clone()));
            }
        }
        return None;
    }

    /* RETURN A JOB REQUEST OF NEXT AVAILABLE REDUCE JOB */
    fn assign_and_ret_reduce_job(&mut self, worker_id: u32) -> Option<(JobRequestReply, u32, SubJob)> { //todo:
        let mut workers_on_job: Vec<u32> = Vec::new();

        for sj in self.map_jobs.iter_mut() {
            let sub_map_job = sj.1;
            assert!(sub_map_job.done);
            /* PUSH THE WORKER WHO WORKED ON THIS JOB */
            workers_on_job.push(sub_map_job.worker_id);
        }

        for sj in self.reduce_jobs.iter_mut() {
            let sub_reduce_job = sj.1;
            if !sub_reduce_job.done && sub_reduce_job.worker_id == 0 {
                let job_req_req = JobRequestReply {
                    file: "".to_string(), // not needed in reduce
                    args: sub_reduce_job.args.clone(),
                    job_type: 1, // 1  is reduce job
                    app: sub_reduce_job.app.to_string(),
                    worker_ids: workers_on_job.clone(),
                    parent_job_id: sub_reduce_job.parent_job_id,
                    job_id: sub_reduce_job.id,
                    reduce_bucket: sub_reduce_job.id,
                    valid: 1,
                    n_reduce: sub_reduce_job.n_reduce,
                    output_dir: sub_reduce_job.output_dir.clone(),
                };
                sub_reduce_job.worker_id = worker_id;
                println!("SENDING REDUCE JOB {} TO WORKER {}", sub_reduce_job.id, worker_id);
                return Some((job_req_req, sub_reduce_job.parent_job_id, sub_reduce_job.clone()));
            }
        }
        return None;
    }
}


pub struct WC {
    worker_id: u32,
    hb_t: u128,
    jobs_done: Vec<SubJob>,
}

impl WC {
    fn update(&mut self, v: u128) {
        self.hb_t = v;
    }
}

#[derive(Default)]
pub struct Coordinator {
    /** WORKER COUNT */
    curr_id: Arc<RwLock<u32>>,
    /** LIST OF ACTIVE WORKERS */
    workers: Arc<RwLock<HashMap<u32, WC>>>,
    /** THE CURRENT JOB COUNT */
    job_cnt: Arc<RwLock<u32>>,
    /** A MAP BETWEEN JOB ID AND THE SUB JOB ID AND THE JOB */
    jobs: Arc<RwLock<HashMap<u32, Job>>>,

}

impl Coordinator {
    pub fn new() -> Self {
        Coordinator {
            curr_id: Arc::new(RwLock::new(0)),
            workers: Arc::new(RwLock::new(HashMap::new())),
            job_cnt: Arc::new(RwLock::new(0)),
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_new_job_id(&self) -> u32 {
        let mut data = self.job_cnt.write().await;
        *data += 1;
        return *data;
    }

    pub async fn add_job(&self, j: Job) {
        let mut jobs = self.jobs.write().await;
        jobs.insert(j.job_id, j);
    }

    pub async fn add_worker(&self) -> u32 {
        let mut data = self.curr_id.write().await;
        *data += 1;
        let mut workers = self.workers.write().await;
        workers.insert(*data, WC { worker_id: *data, hb_t: get_time(), jobs_done: Vec::new() });
        return *data;
    }
}

fn get_time() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    return since_the_epoch.as_millis();
}

fn update_time(w: &mut WC) {
    w.hb_t = get_time();
}

fn dead_worker(worker: &WC) -> bool {
    // println!("time: {}", get_time() - worker.hb_t);
    get_time() - worker.hb_t >= (TASK_TIMEOUT_SECS * 1000) as u128
}

#[tonic::async_trait]
impl coordinator_server::Coordinator for Coordinator {

    async fn submit_job(
        &self,
        req: Request<SubmitJobRequest>,
    ) -> Result<Response<SubmitJobReply>, Status> {
        /** JOB LIST */
        let mut queue = self.jobs.write().await;

        let req = req.into_inner();
        let files = req.files;
        let output_dir = req.output_dir;
        let app = req.app;
        let appli = named(app.clone().as_str());
        match appli {
            Ok(_) => {
                let n_reduce = req.n_reduce;
                let args = req.args;

                let job_id = self.get_new_job_id().await;

                /* MAKE THE JOB */
                let mut job = Job::new(job_id);

                let mut counter_map = 1;
                let mut counter_reduce = 1;
                println!("SUBMITTING JOB {}", job_id);
                /* CREATING MAP JOBS */
                for f in files.iter() {
                    let sub_job_map = SubJob {
                        id: counter_map,
                        parent_job_id: job_id,
                        file: f.to_string(),
                        worker_id: 0,
                        type_job: MapJob,
                        app: app.clone(),
                        n_reduce,
                        args: args.clone(),
                        done: false,
                        failed: false,
                        output_dir: output_dir.clone(),
                    };
                    job.add_job(sub_job_map, MapJob);
                    counter_map += 1;
                }

                /* CREATING REDUCE JOBS */
                while counter_reduce <= n_reduce {
                    let sub_job_reduce = SubJob {
                        id: counter_reduce,
                        parent_job_id: job_id,
                        file: "".parse().unwrap(),
                        worker_id: 0,
                        type_job: ReduceJob,
                        app: app.clone(),
                        n_reduce,
                        args: args.clone(),
                        done: false,
                        failed: false,
                        output_dir: output_dir.clone(),
                    };
                    job.add_job(sub_job_reduce, ReduceJob);
                    counter_reduce += 1;
                }

                /* ADD JOB TO COORDINATOR */
                queue.insert(job_id, job);
                return Ok(Response::new(SubmitJobReply { job_id }));
            }
            Err(e) => return Err(Status::new(Code::NotFound, "JOB APP INVALID"))
        }
    }

    async fn poll_job(
        &self,
        req: Request<PollJobRequest>,
    ) -> Result<Response<PollJobReply>, Status> {
        let req = req.into_inner();
        let j_id = req.job_id;
        let queue = self.jobs.read().await;

        return if queue.contains_key(&j_id) {
            let jobs = queue.get(&j_id).unwrap();
            Ok(Response::new(PollJobReply { done: jobs.done, failed: jobs.failed, errors: jobs.error_arr.clone() }))
        } else {
            Err(Status::new(Code::NotFound, "job id is invalid"))
        };
    }

    async fn register(&self, request: Request<RegisterRequest>) -> std::result::Result<Response<RegisterReply>, Status> {
        let w_id = self.add_worker().await;
        Ok(Response::new(RegisterReply { id: w_id }))
    }

    async fn heart_beat(&self, request: Request<HeartBeatRequest>) -> std::result::Result<Response<HeartBeatReply>, Status> {
        let req = request.get_ref();
        let w_req_id = req.worker_id;
        let mut workers = self.workers.write().await;
        for w in workers.iter_mut() {
            if w_req_id == w.1.worker_id {
                update_time(w.1);
                println!("Updating Worker id: {0}, Worker time: {1}", w.1.worker_id, w.1.hb_t);
            }
        }
        Ok(Response::new(HeartBeatReply {}))
    }


    async fn job_request(&self, request: Request<JobRequestRequest>) -> std::result::Result<Response<JobRequestReply>, Status> {
        let req = request.get_ref();
        let worker_id = req.worker_id;
        let mut jobs = self.jobs.write().await;
        let jreqrep_def = JobRequestReply {
            file: "".to_string(),
            args: vec![],
            job_type: 0,
            app: "".to_string(),
            worker_ids: vec![],
            job_id: 0,
            parent_job_id: 0,
            reduce_bucket: 0,
            valid: 0,
            n_reduce: 0,
            output_dir: "".to_string(),
        };

        /** CHECK FOR DEAD WORKERS AND CHANGE THEIR STATUS OF JOBS */
        let mut workers_list = self.workers.write().await;

        return if jobs.is_empty() {
            /* IF THERE ARE NOT JOBS JUST RETURN A DEFAULT JOB */
            Ok(Response::new(jreqrep_def))
        } else {
            let mut dead_keys = Vec::new();
            for w_entry in workers_list.iter_mut() {
                let w = w_entry.1;
                // println!("worker: {}", w.worker_id);
                if dead_worker(w) {
                    println!("DEAD WORKER: {}, my_id: {}", w.worker_id, worker_id);
                    let mut last_jobs = w.jobs_done.clone();
                    for j in last_jobs.iter_mut() {
                        let parent_job_id = j.parent_job_id.clone();
                        let sub_job_id = j.id.clone();
                        let job_type = j.type_job.clone();

                        let job_net = jobs.get_mut(&parent_job_id).unwrap();
                        if !job_net.done {
                            if job_type == MapJob {
                                let map_net = job_net.map_jobs.get_mut(&sub_job_id).unwrap();
                                map_net.done = false;
                                map_net.worker_id = 0;
                            } else if job_type == ReduceJob {
                                let reduce_net = job_net.reduce_jobs.get_mut(&sub_job_id).unwrap();
                                if !reduce_net.done {
                                    reduce_net.worker_id = 0;
                                }
                            }
                        }
                    }
                    dead_keys.push(w_entry.0.clone());
                }
            }

            for d_k in dead_keys {
                workers_list.remove(&d_k);
            }

            let mut requested_worker = workers_list.get_mut(&worker_id).unwrap();

            /* GET THE KEYS IN SORTED ORDER */
            let all_job_keys = jobs.keys();
            let mut t = Vec::new();
            for k in all_job_keys {
                t.push(k.clone());
            }
            t.sort();
            for k in t {
                let j = jobs.get_mut(&k).unwrap();
                println!("JOB ID {} MAP STATUS {}", j.job_id, j.all_map_done());
                println!("JOB ID {} REDUCE STATUS {}", j.job_id, j.all_reduce_done());

                if j.failed {
                    continue;
                }

                if !j.all_map_done() {
                    let map_job = j.assign_and_ret_map_job(worker_id);
                    /* IF THERE ARE JOBS ARE ASSIGNED */
                    if map_job == None {
                        continue;
                    }
                    let ret = map_job.unwrap();
                    requested_worker.jobs_done.push(ret.2);
                    return Ok(Response::new(ret.0));
                } else if j.all_map_done() && !j.all_reduce_done() {
                    /* DO REDUCE TASK */
                    let reduce_job = j.assign_and_ret_reduce_job(worker_id);
                    /* IF ALL REDUCE JOBS ARE ASSIGNED */
                    if reduce_job == None {
                        println!("REDUCE JOB NONE");
                        continue;
                    }
                    let ret = reduce_job.unwrap();
                    requested_worker.jobs_done.push(ret.2);
                    return Ok(Response::new(ret.0));
                } else {
                    continue;
                }
            }
            Ok(Response::new(jreqrep_def))
        };
    }


    async fn job_done(&self, request: Request<JobDoneRequest>) -> std::result::Result<Response<JobDoneReply>, Status> {
        let req = request.get_ref();
        let worker_id = req.worker_id;
        let job_id = req.job_id;
        let sub_job_id = req.sub_job_id;
        let job_type = req.job_type;

        let mut jobs = self.jobs.write().await;
        let job = jobs.get_mut(&job_id).unwrap();
        let mut workers = self.workers.write().await;
        let w = workers.get_mut(&worker_id).unwrap();
        if job_type == 0 {
            job.map_job_done(sub_job_id);
        } else {
            job.reduce_job_done(sub_job_id);
            job.all_done();
        }
        return Ok(Response::new(JobDoneReply {}));
    }

    async fn connect_error(&self, request: Request<ConnectErrorRequest>) -> std::result::Result<Response<ConnectErrorReply>, Status> {
        let req = request.get_ref();
        let my_id = req.id;
        let red_task_id = req.sub_job_id; // the bucket num or red task id
        let job_id = req.parent_job_id;
        let mut jobs_all = self.jobs.write().await;
        let mut worker_list = self.workers.write().await;
        let me = worker_list.get_mut(&my_id).unwrap();
        let mut my_jobs = &mut me.jobs_done;

        for j in my_jobs {
            if j.parent_job_id == job_id && j.id == red_task_id {
                let r_job_parent = jobs_all.get_mut(&j.parent_job_id).unwrap();
                let r_job = r_job_parent.reduce_jobs.get_mut(&red_task_id).unwrap();
                r_job.worker_id = 0;
            }
        }
        return Ok(Response::new(ConnectErrorReply {}));
    }

    async fn job_failure(&self, request: Request<JobFailureRequest>) -> std::result::Result<Response<JobFailureReply>, Status> {
        let req = request.get_ref();
        let parent_job_id = req.parent_job_id;
        let mut jobs_all = self.jobs.write().await;
        let job_failed = jobs_all.get_mut(&parent_job_id).unwrap();
        job_failed.set_failed(true);
        job_failed.error_arr.push(req.error.clone());
        return Ok(Response::new(JobFailureReply {}));
    }
}

pub async fn start(_args: args::Args) -> Result<()> {
    let addr = COORDINATOR_ADDR.parse().unwrap();
    let mut coordinator: Coordinator = Coordinator::new();
    let svc = coordinator_server::CoordinatorServer::new(coordinator);
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}








