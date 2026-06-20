# Bookrs

This is a simplistic room booking system, where reservations are made at a hotel.

The goal is to allocate guests to the best possible room meeting their constraints.

## The rules
1. If a reservation is made, it must be honored.
2. If a reservation cannot be honored matching its constraints, it can be upgraded but not downgraded.
3. A guest cannot be made to change rooms during their stay.

## The features
In order of importance:
1. Number of nights
1. Number of rooms
2. Class of room (2 queen, 1 king)
3. Adjacency (likely requires a graph implementation)

## Development Plan
1. Proof of Concept (MMVP REPL)
  - This is the initial phase to build a series of dummy pieces so the underlying functionality is present in a basic form.
  - Goal: Demo a room booking system, random allocation, no rebalancing. Basic user functionality.
2. MVP
  - This is where the actual pieces of a meaningful, designed system take place.
  - Frontend: Probably a javascript-based frontend for interacting with a calendar, a dropdown menu for enum values for room type, etc. The first version of this may be a TUI, because I want to get experience with ratatui and building TUIs instead of making an llm write typescript.
  - Backend: a REST API should be sufficient to handle the backend of the web interface. This takes the basics from the proof of concept and stores the hotel state for any given day in a SQL database and provides functionality for querying, making a booking, and returning results to the frontend. For the TUI version, can probably just use the TUI directly with underlying functionality instead of putting an API layer in between.
    - Database: This will probably be sqlite to support an MVP, would migrate it to Postgres for something more performant/scalable.
3. Expansion
    - Develop metrics: Need KPIs to establish a basline for room allocation and to assess improvements.
      - Metric 1: What fraction of available rooms were occupied?
        - Goal: Measure overall room utilization. The more rooms that are utilized, the more money the hotel makes.
      - Metric 2 (rejected): What fraction of potential bookings were accepted?
        - Goal: Accept as many bookings as possible.
        - Reason rejected: This biases toward short bookings of single rooms to bump up this fraction. The goal isn't to accept bookings, it is to maximally utilize the hotel.
    - Add a room rebalancer
    - Implement the REST API architecture if a TUI was implemented to start
    - Add room management functions, e.g. check-in: A guest cannot check in if the room is not ready. There should be a separate API endpoint for marking a room as ready, so cleaning staff can indicate which rooms are available as they go.


### Requirements
1. The rules enumerated above must be followed for a MVP product
2. There is a performance requirement: 
  - Search must be fast so the frontend is snappy; assignment can be slower.
  - The search should be concurrent: multiple users should be able to search efficiently.
  - Assignment should be performed at the time of selection, not payment. This should then be finalized at the payment stage. A daemon can sweep assignments with incomplete payments after some time buffer.
3. Rebalancing should take place to ensure optimal room assignment. This does not need to take place every time a search is executed, but can occur on a cadence (nightly? hourly?). If it takes place during the search stage, it needs to be sufficiently performant as to not impact user experience.
