use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread::spawn;

use rand::Rng;

const NUMBER_OF_THREADS: usize = 10;

fn main() {
    // Whether a cupcake is available or not. The only means of communication between the guests
    let cupcake_on_plate = Arc::new(AtomicBool::new(true));

    // The guests will use this to signal when everyone has entered the labyrinth
    let all_guests_have_entered = Arc::new(AtomicBool::new(false));

    // The means of communication between the Minotaur and the guests. When the Minotaur wants to let one
    // of the guests in they'll send them a signal through one of these senders.
    let mut guest_senders = Vec::with_capacity(NUMBER_OF_THREADS);

    let mut join_handles = vec![];

    // One guest will be the "counter guest" and will keep track of how many guests have visited the labyrinth by counting empty plates.
    // They will be the only one allowed to ask the servants for a new cupcake.
    // They themselves will eat the cupcake once.
    {
        let (counter_guest_sender, counter_guest_receiver) = channel::<Sender<()>>();

        let all_guests_have_entered = all_guests_have_entered.clone();
        let cupcake_on_plate = cupcake_on_plate.clone();

        let mut counter = 0;
        let mut eaten_cupcake = false;

        let counter_guest_handle = spawn(move || loop {
            let finished_sender = counter_guest_receiver.recv().unwrap();

            if all_guests_have_entered.load(Relaxed) {
                // All guests have visited the labyrinth and eaten a cupcake, our work is done
                finished_sender.send(()).unwrap();
                return;
            }

            println!("The counter guest (guest 0) is entering the labyrinth");

            if !cupcake_on_plate.load(Relaxed) {
                // If we encounter an empty plate we that means a new guest has eaten the cupcake
                // We iterate the counter and ask the servants to put a new cupcake on the plate
                counter += 1;
                cupcake_on_plate.store(true, Relaxed);
                println!("The counter guest has encountered an empty plate. {}/{} guests have entered the labyrinth.", counter, NUMBER_OF_THREADS);
            } else if !eaten_cupcake {
                // We should eat the cupcake once ourselves
                cupcake_on_plate.store(false, Relaxed);
                eaten_cupcake = true;
                println!("The counter guest has encountered a cupcake and has eaten it.");
            } else {
                println!("The counter guest has encountered a cupcake but has already eaten one.");
            }

            if counter == NUMBER_OF_THREADS {
                println!(
                    "The counter guest has determined that all guests have entered the labyrinth."
                );
                all_guests_have_entered.store(true, Relaxed);
            }

            // Tell the minotaur we're finished
            finished_sender.send(()).unwrap();
        });

        guest_senders.push(counter_guest_sender);
        join_handles.push(counter_guest_handle);
    }

    // The rest of the guests are responsible for eating the cupcake once
    // Their job is simple: If they see a cupcake and haven't eaten one before they eat it
    for i in 1..NUMBER_OF_THREADS {
        let (guest_sender, guest_receiver) = channel::<Sender<()>>();

        let cupcake_on_plate = cupcake_on_plate.clone();
        let all_guests_have_entered = all_guests_have_entered.clone();

        let mut eaten_cupcake = false;

        let guest_handle = spawn(move || loop {
            let finished_sender = guest_receiver.recv().unwrap();

            if all_guests_have_entered.load(Relaxed) {
                // All guests have visited the labyrinth and eaten a cupcake, our work is done
                finished_sender.send(()).unwrap();
                return;
            }

            println!("A guest (guest {}) is entering the labyrinth", i);

            if cupcake_on_plate.load(Relaxed) {
                if !eaten_cupcake {
                    // Eat the cupcake if we haven't eaten if before
                    cupcake_on_plate.store(false, Relaxed);
                    eaten_cupcake = true;
                    println!("Guest {} encountered and ate the cupcake", i);
                } else {
                    println!(
                        "Guest {} encountered a cupcake but has already eaten one",
                        i
                    );
                }
            } else {
                println!("Guest {} encountered an empty plate", i);
            }

            // Tell the minotaur we're finished
            finished_sender.send(()).unwrap();
        });

        guest_senders.push(guest_sender);
        join_handles.push(guest_handle);
    }

    // Simulate the minotaur letting in one guest at a time
    // The minotaur will let pick a guest at random and let them in
    {
        let all_guests_have_entered = all_guests_have_entered.clone();

        let minotaur_handle = spawn(move || loop {
            if all_guests_have_entered.load(Relaxed) {
                // Once all the guests have entered the labyrinth the minotaur is done sending guests in

                // Inform the guests that the game is finished
                for guest_index in 0..NUMBER_OF_THREADS {
                    let guest_sender = guest_senders.get(guest_index).unwrap();
                    let (finished_sender, finished_receiver) = channel::<()>();
                    guest_sender.send(finished_sender).unwrap();
                    finished_receiver.recv().unwrap();
                }

                return;
            }

            // Send a random guest in and wait for the response
            let mut rng = rand::thread_rng();
            let guest_index = rng.gen_range(0..NUMBER_OF_THREADS);

            println!(
                "The Minotaur has picked guest {} to enter the labyrinth",
                guest_index
            );

            let guest_sender = guest_senders.get(guest_index).unwrap();
            let (finished_sender, finished_receiver) = channel::<()>();
            guest_sender.send(finished_sender).unwrap();

            // Wait for the guest to finish
            finished_receiver.recv().unwrap();
        });

        join_handles.push(minotaur_handle);
    }

    // Wait for the threads to end
    for join_handle in join_handles {
        join_handle.join().unwrap();
    }
}
