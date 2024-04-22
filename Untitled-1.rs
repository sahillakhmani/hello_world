use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::sync::mpsc;

struct Job {
    cost: u32,
}

fn job_generator(tx_scheduler: mpsc::Sender<Job>) {
    let mut rng = thread_rng();
    loop {
        let cost = rng.gen_range(1..=100);
        if cost == 0 {
            // Signaling end of sequence
            break;
        }
        let job = Job { cost };
        tx_scheduler.send(job).unwrap();
    }
}

fn job_scheduler(tx_machine1: mpsc::Sender<u32>, tx_machine2: mpsc::Sender<u32>, rx_scheduler: mpsc::Receiver<Job>) {
    let mut total_cost_machine1 = 0;
    let mut total_cost_machine2 = 0;

    for job in rx_scheduler {
        if total_cost_machine1 <= total_cost_machine2 {
            tx_machine1.send(job.cost).unwrap();
            total_cost_machine1 += job.cost;
        } else {
            tx_machine2.send(job.cost).unwrap();
            total_cost_machine2 += job.cost;
        }
    }
    // Signaling end of processing
    tx_machine1.send(0).unwrap();
    tx_machine2.send(0).unwrap();
}

fn machine(rx_machine: mpsc::Receiver<u32>, report: Arc<Mutex<u32>>) {
    let mut total_cost = 0;
    for cost in rx_machine {
        if cost == 0 {
            break;
        }
        total_cost += cost;
    }
    // Update the total cost in the report
    let mut report = report.lock().unwrap();
    *report += total_cost;
}

fn report_writer(report: Arc<Mutex<u32>>) {
    thread::sleep(Duration::from_secs(2)); // Wait for machines to finish
    let report = report.lock().unwrap();
    println!("Total cost of Machine 1: {}", *report);
    println!("Total cost of Machine 2: {}", *report);
}

