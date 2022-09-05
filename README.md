# Map Reduce using gPRC
Programming Language: Rust
In this project, I implemented the Map Reduce Distributed Computing System using remote procedural calls.
This project was part of a HW section in the CS162 Operating Systems Course at UC Berkeley which I took during the summer of 2022. While this project is fully functioning map reduce system, the majority code for the coordinator and the workers were written by me using the utilities code given by the class. I was part of a few people to have actually completed this homework and receiving 100 percent. In addition, I received an A+ in this class which has a reputation of being one of the most challenging and demanding classes at Berkeley.

![](https://paper-attachments.dropbox.com/s_CBA5DBFEC01DAE4C106C6E17A16C86B1FF78BE19C5CF1F7BF89AB0C450D6AA91_1661462882432_Screen+Shot+2022-08-25+at+2.28.00+PM.png)

# Design and Framework of Jobs from Coordinator Perspective

To construct the coordinator, I decided to move forward with the following structures that hold the metadata of the job that has been submitted. In addition, the coordinator needs to keep track of the completion of the job and of any hazards and errors that happen as the job is executed.


    pub struct Job {
        /** JOB ID */
        job_id: u32,
        /** MAP JOB ID AND THE MAP JOBS */
        map_jobs: HashMap<u32, SubJob>,
        /** REDUCE JOB ID AND THE REDUCE JOB */
        reduce_jobs: HashMap<u32, SubJob>,
        /** WHETHER THE JOB HAS COMPLETED */
        done: bool,
        /** WHETHER THE JOB HAS FAILED */
        failed: bool,
        /** THE ERROR ASSOCIATED WITH A ERRORED JOB */
        error_arr: Vec<String>,
    }

The `pub struct job` has fields that keep track of the job execution. Each job is split up into their respective `map_jobs` and `reduce_jobs` which are hash maps with their id and the `SubJob` which could be a map or a reduce job.


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

The `pub struct SubJob` has fields that keep track of the execution of the map or reduce job.
To know whether a map job has failed or a reduce job has failed and propagate it to the `struct Job` and to re-spawn this specific job if a worker fails, these are some of the primary functions of this struct.

# Design and Framework of Workers From Coordinator Perspective

The coordinator must have a way to keep track of workers that are alive or dead. In addition, if a worker dies mid execution, the coordinator must re-route the sub job to another alive worker that is free. To facilitate this a worker structure that keeps track of workers is necessary.


    pub struct WC {
        worker_id: u32,
        hb_t: u128,
        jobs_done: Vec<SubJob>,
    }

Since the coordinator executes tasks in a concurrent environment all the fields that are shared across workers and the coordinator must be mutex.


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


# Design and Framework of Jobs from Worker Perspective

As a worker, there are fewer things to keep track of namely. Namely, the id assigned by the coordinator and the jobs list that the worker has executed. Each worker has a jobs list that is wrapped in a mutex since it is a shared object. The sharing occurs when the requesting worker is working on a reduction job that needs map data from the map job executed by the requested worker.


    pub struct Worker {
        /** WORKER ID ASSIGNED BY THE COORDINATOR */
        id: u32,
        /** JOB ID AND THE CORRESPONDING SUB JOB ID AND ITS BUCKET */
        jobs: Arc<Mutex<HashMap<u32, HashMap<u32, Vec<bytes::Bytes>>>>>,
    }
# The Logic
## Submit Job From the Coordinator Perspective
- The user submits the job using the remote procedure call `async fn submit_job` which takes in the following fields.
- The protocol buffer for the submit job is as follows:
  message SubmitJobRequest {
  repeated string files = 1;
  string output_dir = 2;
  string app = 3;
  uint32 n_reduce = 4;
  bytes args = 5;
  }
  message SubmitJobReply {
  uint32 job_id = 1;
  }
- The user submits all the necessary information including the files, the output directory, the application, the number of reduction buckets, and also any optional argument. The coordinator replies with the assigned job id that is unique.
- When the submission is in process, the coordinator does some bookkeeping by constructing all the `SubJob` associated with this `Job`. Pushing them onto its mutex protected `jobs` field.
- The `SubJob` is map and reduce tasks.
## Worker Registration From the Coordinator Perspective
- When a worker is spawned, the worker using a RPC registers as a worker.
  pub struct WC {
  worker_id: u32,
  hb_t: u128,
  jobs_done: Vec<SubJob>,
  }
- The coordinator constructs the worker `struct WC` updates the fields and adds it to the field `workers` that is map between a unique worker id and the object.
- In addition, the worker calls a `send_heart_beat` function which in turn spawns a new async thread which calls the `heart_beat` rpc to let the coordinator update the time signature of the worker. This is done to ensure that the coordinator can keep track of which workers are alive and dead.
## Worker Job Request From the Coordinator Perspective
- After a worker has registered and determined alive, the worker uses an RPC to request a job. As an illustration, this is the protocol buffer for job request.
  message JobRequestRequest {
  uint32 worker_id = 1;
  }

  message JobRequestReply {
  string file = 1;
  bytes args = 2;
  uint32 job_type = 3;
  string app = 4;
  repeated uint32 worker_ids = 5;
  uint32 parent_job_id = 6;
  uint32 job_id = 7;
  uint32 reduce_bucket = 8;
  uint32 valid = 9;
  uint32 n_reduce = 10;
  string output_dir = 11;
  }
- The coordinator finds the next available map sub job or reduce sub job--if all map sub jobs for a particular job are completed—and sends that information to the worker.
- Note, the map sub jobs are prioritized by the order of job id, i.e, jobs that had been submitted earlier get priority.
## Worker Job Request From the Worker Perspective
- Once the worker has received the information necessary for the job, it proceeds with the execution of the job.
- If it is a map job, it proceeds to work on the call `do_map_job` which does the map job.
    - As the map job begins executing, the worker begins by first finding the buckets necessary for this job, and if such buckets don’t exist, it inserts one into the current `Worker` `jobs` map.
    - Note, the buckets is a hash map as well. A hash map between bucket number and the bucket.
      /** GET THE BUCKETS CORRESPONDING TO A JOB */
      let buckets = jobs.entry(parent_job_id).or_insert(HashMap::new());
    - Next, the worker,  opens the input file, reads the contents, and forms a key value pair object where the key is the input file name and the value is the content of the file.
      let kv = KeyValue {
      key: bytes::Bytes::from(input_file),
      value: content,
      };
    - The worker then performs the map operation of the key value object and the auxiliary function.
    - Once the map operation is performed, the worker hashes the output key to find the bucket number, the bucket, and pushes the output value as the bucket data.
    - This map data will be used during the reduction process.
    - Once the map job is done, the worker uses an RPC to let the coordinator know that the map job, by calling `job_done` RPC, has been successfully completed, and if not, calls the `job_failure` RPC and sends the error messages.
- If it is a reduction sub job, the worker does the following.
    - It proceeds by first caching all the worker ids that did the map sub jobs for this particular job.
    - While iterating through every single worker id, the reduce worker does a data request to receive the map data from that particular worker.
    - The reduce worker does this using a worker RPC.
      message MapDataRequestRequest {
      uint32 job_id = 1;
      uint32 bucket_num = 2;
      bytes args = 3;
      string app = 4;
      string output_dir = 5;
      }

  message MapDataRequestReply {
  bytes data = 1;
  }
    - Once the data is acquired, the reduce worker begins by unpacking the map data from all the workers.
    - Once the data is unpacked and all the values are added to a list structure for a particular key, the reduce worker calls the reduce function on this values list for every key.
    - The reduce worker then writes the reduced data into a file.
    - Finally, the reduce worker calls the `job_done` coordinator RPC to let the coordinator know the reduce job has been completed successfully, and if not calls the `job_failure` coordinator RPC to let the coordinator know of the error messages.
# In Action

To run,

- cargo build
- cd ./target/debug

To run coordinator,

- ./mr-coordinator

To run worker,

- ../mr-worker

To submit example job,

- ../mr-client submit --output-dir out --app wc ../../data/gutenberg/p.txt

To retrieve processed data

- ./mr-client process --output-dir out --app wc

Example with one coordinator, two workers, and one word count job

![](https://paper-attachments.dropbox.com/s_CBA5DBFEC01DAE4C106C6E17A16C86B1FF78BE19C5CF1F7BF89AB0C450D6AA91_1662348575735_Screen+Shot+2022-09-04+at+8.29.33+PM.png)

![](https://paper-attachments.dropbox.com/s_CBA5DBFEC01DAE4C106C6E17A16C86B1FF78BE19C5CF1F7BF89AB0C450D6AA91_1662348602366_Screen+Shot+2022-09-04+at+8.30.00+PM.png)

![](https://paper-attachments.dropbox.com/s_CBA5DBFEC01DAE4C106C6E17A16C86B1FF78BE19C5CF1F7BF89AB0C450D6AA91_1662348622523_Screen+Shot+2022-09-04+at+8.30.19+PM.png)

![](https://paper-attachments.dropbox.com/s_CBA5DBFEC01DAE4C106C6E17A16C86B1FF78BE19C5CF1F7BF89AB0C450D6AA91_1662348637260_Screen+Shot+2022-09-04+at+8.30.35+PM.png)

![](https://paper-attachments.dropbox.com/s_CBA5DBFEC01DAE4C106C6E17A16C86B1FF78BE19C5CF1F7BF89AB0C450D6AA91_1662348671817_Screen+Shot+2022-09-04+at+8.31.09+PM.png)

![](https://paper-attachments.dropbox.com/s_CBA5DBFEC01DAE4C106C6E17A16C86B1FF78BE19C5CF1F7BF89AB0C450D6AA91_1662348855352_Screen+Shot+2022-09-04+at+8.34.10+PM.png)


