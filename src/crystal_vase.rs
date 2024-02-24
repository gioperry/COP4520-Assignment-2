use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread::spawn;

use rand::Rng;

const NUMBER_OF_THREADS: usize = 10;

struct Guest {
    entrance_sender: Sender<()>,
}

impl Guest {
    fn send_inside(&self) {
        self.entrance_sender.send(()).unwrap();
    }
}

fn main() {
    // Represents the position of the guests in the queue to see the crystal vase
    let queue = Arc::new(Mutex::new(Vec::with_capacity(NUMBER_OF_THREADS)));

    // A way to lookup guests by id (index of the guest list)
    let guest_list = Arc::new(Mutex::new(Vec::<Guest>::with_capacity(NUMBER_OF_THREADS)));

    let mut join_handles = Vec::with_capacity(NUMBER_OF_THREADS);

    // Spawn the guest threads & make a guest list
    for guest_id in 0..NUMBER_OF_THREADS {
        let (entrance_sender, entrance_receiver) = channel::<()>();

        let guest_local_queue = queue.clone();
        let guest_local_guest_list = guest_list.clone();

        let join_handle = spawn(move || loop {
            // Wait for a signal to enter the showroom
            let _entry_signal = entrance_receiver.recv().unwrap();

            visit_showroom(guest_id);

            // Simulate a guest deciding to get back into the queue
            let mut got_back_in_line = false;

            if random_bool() {
                let mut queue_guard = guest_local_queue.lock().unwrap();
                queue_guard.push(guest_id);
                got_back_in_line = true;
            }

            // Notify the next guest in line that they can enter
            let mut queue_guard = guest_local_queue.lock().unwrap();

            if !queue_guard.is_empty() {
                let next_guest_id = queue_guard.remove(0);
                let guest_list = guest_local_guest_list.lock().unwrap();
                let next_guest = guest_list.get(next_guest_id).unwrap();
                next_guest.send_inside();
            }

            if got_back_in_line {
                println!("Guest {} has decided to get back in line", guest_id);
            } else {
                // If we didn't get back in line we have nothing else to do
                return;
            }
        });

        let guest = Guest { entrance_sender };

        let mut guest_list_guard = guest_list.lock().unwrap();
        guest_list_guard.push(guest);

        join_handles.push(join_handle);
    }

    // Populate the queue of guests
    let mut queue_guard = queue.lock().unwrap();
    for guest_id in 0..NUMBER_OF_THREADS {
        queue_guard.push(guest_id);
    }

    // Tell the first guest to enter the room
    let first_guest_id = queue_guard.remove(0);
    let guest_list_guard = guest_list.lock().unwrap();
    let first_guest = guest_list_guard.get(first_guest_id).unwrap();
    first_guest.send_inside();

    // Drop references so the guests can access them
    drop(queue_guard);
    drop(guest_list_guard);

    // Wait for all the guests to finish
    for join_handle in join_handles {
        join_handle.join().unwrap();
    }

    println!("All the guests have finished viewing the vase");
}

fn visit_showroom(guest_id: usize) {
    println!("Guest {} is visiting the showroom", guest_id);
}

fn random_bool() -> bool {
    let mut rng = rand::thread_rng();
    rng.gen_bool(0.5)
}
