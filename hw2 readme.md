
## To compile & run problem 1

```bash
cargo run --bin birthday_party --release
```

## To compile & run problem 2

```bash
cargo run --bin crystal_vase --release
```


## Problem 1
- I decided to use queues to implement the solution to this problem. Each guest is given a queue that only the Minotaur can add to. The queues are stored in a list and given to the Minotaur so he can communicate with each guest by index.
- The guest threads run in a loop where they wait for the Minotaur to add something to the queue. This is the guest's signal to enter the labyrinth. It's up to the Minotaur thread to make sure only one guest enters the labyrinth at a time as they have the power to let all the guests in at once.
- The Minotaur doesn't just add any arbitrary value to the queue though - they add a reference to a temporary queue (`Sender`) that the Minotaur creates when sending a guest in. The Minotaur waits for an empty value (`()`) to be added to this queue before letting another guest in.
- The state of the cupcake is stored in an atomic boolean and shared between all guests.

## Problem 2 

#### Picking a solution
- Solution 1 has a glaring issue. Guests aren't guaranteed to be able to see the vase since there could be greedy people that rush for the door as soon as its open.
- I honestly can't figure out what the difference is between Solution 1 and Solution 2. It seems like the occupant is responsible for putting up a flag (closing the door or putting up a sign) in both scenarios and any other guest can check the flag freely.
- Solution 3 sounds like the most efficient & fair solution since a guest will almost always be occupying the storeroom. Once a guest exits he notifiers the next in line and they go right in. The problem with this solution is the guests don't get to enjoy the party while waiting, they're stuck in line. I decided to implement this solution.
#### Implementing the solution
- I decided to use queues again for this one. Each guest thread has a queue that they wait for a message on before entering the showroom.
- Once a guest has entered the showroom they pick the next guest in line and send them a message through their queue.
- When each guest has entered the showroom there's a random chance that they insert themselves back into the queue.
- If the guest got back into the queue the guest waits for another entrance message
