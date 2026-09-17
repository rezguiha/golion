# Context:
One of the open subjects about the bidding strategies to implement in this project was whether we would allow an asset in a specific market to bid in markets of other countries or not.
The original design was leaning more into enabling it.

# Decision 16/09/2026

It has been decided to stick only to asset country market bidding for the following reasons:
- Even though wholesale markets enable this some limitations are imposed on it. -> Needs to be modeled
- Cross country wholasale market bidding needs to take into account congestions and losses-> needs to be modeled
- Even though platforms like Picasso and Mari enable it there are so many unknowns about such bidding and the national priority
concepts.

My judgement at this point is that it introduces too much complexity and grey areas. It is more relevant to focus
on asset country markets bidding.

This will also enable us to decouple and isolate optimizations per country and trigger optimizations per country in parallel.
