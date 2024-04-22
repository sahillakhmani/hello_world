use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use rand::{thread_rng, Rng};

struct Job {
    cost: u32,
}

fn job_generator(tx_scheduler: mpsc::Sender<Job>, tx_end: mpsc::Sender<()>) {
    let mut rng = thread_rng();
    loop {
        let cost = rng.gen_range(0..=100);
        let job = Job { cost };
        if cost == 0 {
            // Signaling end of sequence
            tx_end.send(()).unwrap();
            break;
        }
        tx_scheduler.send(job).unwrap();
    }
}

fn job_scheduler(tx_machine1: mpsc::Sender<u32>, tx_machine2: mpsc::Sender<u32>, rx_scheduler: mpsc::Receiver<Job>, rx_end: mpsc::Receiver<()>) {
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

    // Wait for the end of job generation
    rx_end.recv().unwrap();

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

fn report_writer(report_machine1: Arc<Mutex<u32>>, report_machine2: Arc<Mutex<u32>>) {
    let report1 = report_machine1.lock().unwrap();
    let report2 = report_machine2.lock().unwrap();
    println!("Total cost of Machine 1: {}", *report1);
    println!("Total cost of Machine 2: {}", *report2);
}

fn main() {
    let (tx_scheduler, rx_scheduler) = mpsc::channel();
    let (tx_machine1, rx_machine1) = mpsc::channel();
    let (tx_machine2, rx_machine2) = mpsc::channel();
    let (tx_end, rx_end) = mpsc::channel();
    let report_machine1 = Arc::new(Mutex::new(0));
    let report_machine2 = Arc::new(Mutex::new(0));
    let barrier = Arc::new(std::sync::Barrier::new(5));

    let job_generator_thread = thread::spawn({
        let barrier = Arc::clone(&barrier);
        move || {
            job_generator(tx_scheduler, tx_end);
            barrier.wait();
        }
    });

    let job_scheduler_thread = {
        let tx_machine1 = tx_machine1.clone();
        let tx_machine2 = tx_machine2.clone();
        thread::spawn({
            let barrier = Arc::clone(&barrier);
            move || {
                job_scheduler(tx_machine1, tx_machine2, rx_scheduler, rx_end);
                barrier.wait();
            }
        })
    };

    let machine1_thread = {
        let report_machine1 = Arc::clone(&report_machine1);
        thread::spawn({
            let barrier = Arc::clone(&barrier);
            move || {
                machine(rx_machine1, report_machine1);
                barrier.wait();
            }
        })
    };

    let machine2_thread = {
        let report_machine2 = Arc::clone(&report_machine2);
        thread::spawn({
            let barrier = Arc::clone(&barrier);
            move || {
                machine(rx_machine2, report_machine2);
                barrier.wait();
            }
        })
    };

    let report_writer_thread = {
        let barrier = Arc::clone(&barrier);
        let report_machine1 = Arc::clone(&report_machine1);
        let report_machine2 = Arc::clone(&report_machine2);
        thread::spawn(move || {
            barrier.wait(); 
            report_writer(report_machine1, report_machine2);
        })
    };

    job_generator_thread.join().unwrap();
    job_scheduler_thread.join().unwrap();
    machine1_thread.join().unwrap();
    machine2_thread.join().unwrap();
    report_writer_thread.join().unwrap();
}
