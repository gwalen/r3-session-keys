# R3 Session Keys Home assignment

## Framework choice
This program uses Anchor version 1.1.2, the latest stable Anchor release at the time of writing. Anchor is the most widely adopted Solana program framework and 
has an established ecosystem, documentation, tooling, and developer community.
It abstracts low-level operations such as account validation, serialization, PDA constraints, CPI construction, IDL generation, and client generation,
allowing development to focus on business logic. 

Framework selection should also consider long-term maintenance. A widely adopted framework makes it easier for other developers to understand,
review, and contribute to the program. Selecting a niche framework may introduce risks such as limited documentation, a smaller contributor pool, unstable APIs, or eventual discontinuation.

I have considered the following alternatives:

- This framework is gaining popularity and uses zero-copy, no_std, and other optimizations that significantly reduce program binary size and CU usage. Pinocchio and P-Token were audited.
Pinocchio is also a low-level framework. It requires more boilerplate and sometimes uses Rust unsafe, so it has a much higher entry level than Anchor.
Moving to Pinocchio could be considered as an optimization after the initial phase of testing with partners, if reducing binary size and CU usage becomes a priority.

- Steel: This is another low-level framework that can reduce binary size and CU usage. 
  However, it has a smaller maintainer and developer community,remains unaudited, and is still a relatively niche project.

- Anchor V2: This is the next version of Anchor, based on Pinocchio. It provides significant reductions in binary size and CU usage while still abstracting low-level operations similarly to Anchor V1.
  At the time of writing, it is not yet ready for production because it has only recently completed an audit and reached RC-1 version.
  It could become the best choice once a stable version is released.

- Quasar: This is an Anchor V2 competitor developed by Blueshift. 
  It is still in beta and unaudited, so it is not yet production-ready. A stable release may make it worth reconsidering later.  

