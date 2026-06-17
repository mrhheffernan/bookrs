# Bookrs

This is a simplistic room booking system, where reservations are made at a hotel.

The goal is to allocate guests to the best possible room meeting their constraints.

The rules:
1. If a reservation is made, it must be honored.
2. If a reservation cannot be honored matching its constraints, it can be upgraded but not downgraded.
3. A guest cannot be made to change rooms during their stay.

The constraints, in order of importance:
1. Number of nights
1. Number of rooms
2. Class of room (2 queen, 1 king)
3. Adjacency (likely requires a graph implementation)