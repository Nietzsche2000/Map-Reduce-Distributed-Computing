//! Basic MapReduce tests.
//!
// Feel free to modify this file;
// tests here are NOT run by the autograder.

use std::time::Duration;

use crate::client::*;
use crate::rpc::coordinator::*;
use crate::utils::start_cluster;
use crate::COORDINATOR_ADDR;
use anyhow::Result;
use log::{log, Level};

/// An example test that you can run locally.
/// This test starts a cluster of 4 workers,
/// submits a word count job to the coordinator,
/// and waits for the job to complete.
/// It then prints the final output from running word count.
///
/// The autograder runs more detailed tests.
/// This test likely won't pass until you implement task execution.
#[tokio::test]
// #[ignore]
async fn test_wc() -> Result<()> {
    println!("running test");
    let n_workers = 20;
    start_cluster(n_workers).await;

    let mut client =
        coordinator_client::CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR))
            .await?;

    let infile = "./data/gutenberg/whale.txt".to_string();
    // let infile = "./data/gutenberg/p.txt".to_string();
    // let infile2 = "./data/gutenberg/q.txt".to_string();
    // let infile3 = "./data/gutenberg/r.txt".to_string();
    // let infile4 = "./data/gutenberg/s.txt".to_string();
    // let infile5 = "./data/gutenberg/t.txt".to_string();
    // let infile6 = "./data/gutenberg/u.txt".to_string();
    // let infile5 = "./data/alphabet2/letters2.txt".to_string();
    let output_dir = "/tmp/hw-map-reduce/test-wc".to_string();
    // let output_dir2 = "/tmp/hw-map-reduce/test-wc2".to_string();
    let n_reduce = 5;
    let job_id = submit_job(
        &mut client,
        // vec![infile.clone(), infile2.clone(), infile3.clone(), infile4.clone(), infile5.clone(), infile6.clone()],
        vec![infile.clone()],
        output_dir.clone(),
        "wc".to_string(),
        n_reduce,
        vec![0, 1, 2, 3],
    )
        .await?;

    // let job_id2 = submit_job(
    //     &mut client,
    //     vec![infile.clone(), infile2.clone(), infile3.clone(), infile4.clone(), infile5.clone(), infile6.clone()],
    // vec![infile.clone()],
    // output_dir2.clone(),
    // "wc".to_string(),
    // n_reduce,
    // vec![0, 1, 2, 3],
    // )
    //     .await?;

    let mut res = poll_job(&mut client, job_id).await?;
    // loop {
    //     res = poll_job(&mut client, job_id).await?;
    //     println!("job status {}", res.done);
    //     if res.done {
    //         break;
    //     }
    // }
    while {
        res = poll_job(&mut client, job_id).await?;
        !(res.done || res.failed)
    } {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }


    assert!(res.done && !res.failed);

    let s = postprocess_job(&output_dir, "wc", n_reduce)
        .await
        .expect("failed to postprocess");

    // println!("{}", s);

    Ok(())
}

#[tokio::test]
async fn test_vertex() -> Result<()> {
    println!("running test");
    let n_workers = 3;
    start_cluster(n_workers).await;

    let mut client =
        coordinator_client::CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR))
            .await?;

    let infile = "./data/graph-edges-medium/00.txt".to_string();
    let infile1 = "./data/graph-edges-medium/01.txt".to_string();
    let infile2 = "./data/graph-edges-medium/02.txt".to_string();
    let infile3 = "./data/graph-edges-medium/03.txt".to_string();
    let infile4 = "./data/graph-edges-medium/04.txt".to_string();

    let output_dir = "/tmp/hw-map-reduce/test_vertex".to_string();
    let n_reduce = 5;
    let job_id = submit_job(
        &mut client,
        vec![infile.clone(), infile1.clone(), infile2.clone(), infile3.clone(), infile4.clone()],
        output_dir.clone(),
        "vertex-degree".to_string(),
        n_reduce,
        vec![],
    )
        .await?;

    let mut res = poll_job(&mut client, job_id).await?;
    loop {
        res = poll_job(&mut client, job_id).await?;
        println!("job status {}", res.done);
        if res.done {
            break;
        }
    }
    // while {
    //     res = poll_job(&mut client, job_id).await?;
    //     !(res.done || res.failed)
    // } {
    //     tokio::time::sleep(Duration::from_secs(1)).await;
    // }


    assert!(res.done && !res.failed);

    let s = postprocess_job(&output_dir, "vertex-degree", n_reduce)
        .await
        .expect("failed to postprocess");

    println!("{}", s);

    Ok(())
}

#[tokio::test]
async fn test_grep() -> Result<()> {
    println!("running test");
    let n_workers = 3;
    start_cluster(n_workers).await;

    let mut client =
        coordinator_client::CoordinatorClient::connect(format!("http://{}", COORDINATOR_ADDR))
            .await?;

    let infile = "./data/gutenberg/p.txt".to_string();
    let output_dir = "/tmp/hw-map-reduce/test_grep".to_string();
    let n_reduce = 5;
    let job_id = submit_job(
        &mut client,
        vec![infile.clone()],
        output_dir.clone(),
        "grep".to_string(),
        n_reduce,
        Vec::from("hello"),
    )
        .await?;

    let mut res = poll_job(&mut client, job_id).await?;
    // loop {
    //     res = poll_job(&mut client, job_id).await?;
    //     println!("job status {}", res.done);
    //     if res.done {
    //         break;
    //     }
    // }
    while {
        res = poll_job(&mut client, job_id).await?;
        !(res.done || res.failed)
    } {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }


    assert!(res.done && !res.failed);

    let s = postprocess_job(&output_dir, "vertex-degree", n_reduce)
        .await
        .expect("failed to postprocess");

    println!("{}", s);

    Ok(())
}