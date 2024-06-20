![DFE Org](https://github.com/osmosis-labs/defe/assets/43707795/48f93573-759b-4d45-a0a7-508f7b8b29e1)

# Decentralizing the Osmosis Frontend

## 1. System Architecture

### 1.1 Decentralized Domain Registry
- Optimism is already acting as our blockchain-based registry for managing .box domains and their ownership.

### 1.2 Upgrading Osmosis to Include SGX Enclaves
- Secure execution environments for hosting the Osmosis front end code and managing DNS resolution.

### 1.3 SGX MPC Networking
- A network of nodes participating in the multi-party computation for key management and custody.
- The approach of using SGX's multi-signed MPC TLS key signature for verifying domain ownership is valid.
- SGX's ability to provide a trusted execution environment (TEE) and sign data using a key that never leaves the enclave ensures the integrity and authenticity of the data and code.

### 1.4 Cloudflare Integration Preference
- Incorporating Cloudflare services for DDoS protection, real-time updates, and enhanced DNS resolution.

### 1.5 HTTPS Enforcement
- Implementing HSTS.

### 1.6 Thin Client Application
- A user-friendly application for interacting with the decentralized frontend, handling installation, updates, and communication with SGX enclaves.

### 1.7 Governance Framework
- A decentralized governance system using tools like Aragon or Gnosis for managing the .box domain and making decisions related to the frontend.
- We should even explore Cosmos DAO that has a focus on DAO Key management and web working.

### 1.8 Validator Set Key Management
- The validator set should be able to own the key for the .box domain, using SGX's / MPC.

### 1.9 Validator Responsibilities
Validators should be able to (in either SGX, MPC, or both):
- Create TLS certificates
- Revoke TLS certificates
- Update DNS settings 
- Set the Code hash for what SGX code can run the FE 
- Write SGX code that runs an Osmosis Light Client + Serves FE 
- Serve the FE code based upon a particular code hash / IPFS link osmosis governance chooses 
- Use the TLS key communicated from the prior step

### 1.10 Guidelines for DOS Resistance
- We cannot use app-layer cloudflare DOS guarantees.
- We need to evaluate vulnerability, and if so, come up with app-layer protections [1.6].
- SGX operators can still use off-the-shelf network layer DOS protections.

### 1.11 Find an Architecture to Get Validators Provably Running FE-serving SGX's

### 1.12 Get All of the FE Data Dependencies Verified (Separate Step/Project)

## 2. System Workflow

### 2.1 Decentralized Javascript Delivery
1. Users access the Osmosis frontend using the osmosis.box domain. 
2. DNS resolution points to {some server}, this step can only lead to liveness, not correctness failures. (Liveness failure risk is true of the entire internet) 
3. The user does a TLS handshake with the server (same as any website). Only SGX's running the correct code could have the TLS certificate key, hence the user is talking to the correct code being served by an enclave. 
4. The front end code is served from SGX enclaves, ensuring its integrity and authenticity. 
5. HTTPS is enforced for every future connection, using HSTS.

### 2.2 Dependency Verification
Options:
- Server Side SGX, client checks SGX proof.
- Untrusted server sends something that's inherently verifiable ("NP") to the client, and the client checks it. (e.g., Routing) 
- Untrusted server sends something with ZKP directly, client checks ZKP 
- zk-Lightweight calculations and validations are performed on the client-side, while heavy tasks are delegated to server-side environments or specialized cloud services. 
- ZKPs and recursive SNARKs are used to verify the integrity and correctness of routes and computations. 
- The client-side application provides a seamless user experience, handling any remaining installation, updates, and communication with SGX enclaves.
- Governance decisions related to the frontend are made through the decentralized governance framework. 
- Charter that dictates code governance.

## 3. Challenges

### Challenge #1: Utilization of the .box Domain for the Osmosis Frontend
- **Question**: How can the Osmosis community acquire and manage a DNS handle?
- **Action Plan**: Register for the ENS handle "osmosis.BOX". The key owning this handle can be community-owned, with the community pool possessing an MPC secret that rotates on the Ethereum/Optimism mainnet every month. We have a longer-term plan of utilizing SGX to pass the key in a much more efficient way.
- **Management Strategy**: It's likely we will manage this with a multisig wallet. First, we already acquired the "osmosis.box" domain, and we think we can then map it to the Osmosis frontend. The multisig wallet will then own the .box domain, establishing governance over the wallet that holds the key. Thereafter, the signature will be transferred to Optimism for managed key oversight.
- **Tool Consideration**:
  - What DAO tools can Osmosis utilize to manage this process, considering it's part of a separate ecosystem?
  - **Idea**: Tools like Aragon or Gnosis could be integrated to facilitate governance and operational procedures within the Decentralized Front end 'DAO', ensuring transparent and democratic management practices. https://handshake.org/ can be used as our CA for the og tls certification. 
  - Before we move away from Optimism to Osmosis, what would these governance practices look like? Whatever this process would be, we'd then want to push this over to the SGX after a DAO manages the control of the domain.
- **Current Scenario**: 
  - The "osmosis.limo" standard in Ethereum manages domain names but is perceived as suboptimal due to its URL format. This is why we're considering moving to .box for Osmosis.

### Challenge #2: Long-term Decentralization of DNS-to-IP Mapping
- **Proposed Long-term Solutions**: Consider utilizing SUAVE [3] or Multi-Party Computation (MPC), where DNS ownership lies within an MPC managed by validators on Osmosis. We need more concrete solutions, but it appears SUAVE could readily support this functionality. DAO key management to employ Software Guard Extensions (SGX) or MPC custodianship by the validator set for secure and decentralized DNS management.
- **Idea**: a proposed system for secure and decentralized domain ownership verification and DNS management using Multi-Party Computation (MPC) within Intel SGX enclaves, with potential long-term solutions leveraging SUAVE, MPC. or QUIC.

### Challenge #3: Verifiably Correct Code Delivery
- **Problem Statement**: How can we ensure that the server content is verifiably "correct" (as approved by Osmosis governance) and decentralized (not reliant on a single company's cloud infrastructure)?
- **Constraint**: The system currently relies on server-side logic for rendering and basic calculations.
- **Proposed Solution**: Utilize Flashbots' SUAVE to enable SGX to serve this part of the frontend. They have developed a proof of concept for dynamically adding SGX to run the verified code in DNS mappings. "We are collaborating with Xyn from Flashbots on a blog post about this solution for decentralized front-ends." – Dev Ojha
- **Implementation Strategy**:
  - The solution divides into two primary approaches:
    1. Routing: We would request specific routes from the server.  
    2. Verification of Computational Problems: The nature of problems handled by the client's browser should be those that do not require heavy-duty rendering of complex algorithms.
  - Users perform lightweight calculations, and we restrict the client-side computations to tasks that don't require intensive CPU usage. Perform simple validations locally, such as checking if the user inputs are within acceptable ranges or formats before submitting them to the server or blockchain. Optimization problems, often NP-hard, meaning they are quick to verify but potentially slow to solve optimally. We would consider using 'Web Workers' in JavaScript, which allows the heavy tasks to run in a separate thread without freezing the user interface and delegating heavy mathematical computations (e.g., predictive modeling) tasks to powerful server-side environments or specialized cloud services.
  - Volume Adjustments: Calculating the adjusted amounts of tokens a user wants to swap or receive based on current exchange rates or liquidity pool depths.
  - Slippage Calculations: Determining the expected slippage for trades based on pool size and the trade amount.
  - Fee Estimations: Quickly estimating transaction fees based on predefined network conditions or user-selected priorities.
- To optimize the client-side performance, follow best practices in asynchronous execution / incremental loading methods. Even if a task takes a considerable amount of time, progress indicators and client-side caching help supplement client to server-side task management operations.  
- For verifying the integrity and correctness of the routes and computations, instead of using computationally intensive methods like SNARKs, we would utilize inherently verifiable NP problem sets. 
- The process involves querying the server, whereupon the light client ingests a zero-knowledge proof (ZKP) of the query. Subsequently, a recursive SNARK is applied to this data, facilitating a live query that can accurately assess and verify total volumes. To achieve a verifiable index, tools like Axiom are employed, ensuring that the data integrity and correctness are maintained throughout the process. 
- The light client should receive a ZKP for the queried data. This proof allows the client to verify the correctness of the data without seeing the actual data or requiring the storage of the complete dataset. 
- A (SNARK) would allow compressing multiple proofs into a single one, making the verification process more efficient and less data-intensive. The implementation of recursive SNARKs is optimized to handle the expected load, like hardware acceleration or libraries like SP1 could end up coming really handy for something like this. The Recursive SNARKs are applied to the ZKP received from the SGX server verifiably hosting the application and enabling the light client to efficiently verify a chain of proofs, culminating in a live query that assesses calculations such as the list described prior (e.g., total volumes, fees, slippage estimates). 
- Axiom can be used to create a verifiable index, which helps in organizing and ensuring the integrity of the data stored on-chain. This tool allows users and applications to verify that the data they are querying is correct and unchanged. 
- And then this – https://docs.farcaster.xyz/learn/what-is-farcaster/frames 

### Challenge #4: Ensuring Data Dependency Verification
- **Objective**: Guarantee that all "data dependencies" are verified, covering current requirements and routing simplifications.
- **Approach**: Develop and implement strong verification processes to ensure that all data dependencies, whether complex or straightforward, are consistently and reliably verified within the system.
  1. First, we check the SGX's multi-signed MPC TLS key signature for verifying domain ownership.
  2. There is a public key that is only stored in SGX running correct code & hash. Using a clever key supply chain verifying signature from such public key authentication that this is signed by SGX.
  3. SGX can provide a trusted execution environment (TEE) where code runs in an isolated manner, preventing unauthorized access even from the host machine. SGX's ability to sign data using a key that never leaves the enclave (isolated execution environment) ensures that any data or code coming out of SGX can be verified for integrity and authenticity.
- **Correctly Mapping Osmosis.box Domains to its appropriate IP decentralized authoritative nameserver**:
  - "Bob in Berkeley wants to visit osmosis.box. How do we map the osmosis.box to the IP address?" – The IP's set by Optimism ([dot] box Multisig owner) private key owner.
  - The user types osmosis.box into the browser, and the browser first checks its cache. If there are no answers there, it makes an OS call to try to get the answer. The OS call likely has its own cache as well, but if the answer is not there either, it reaches out to the DNS resolver. The DNS resolver first checks its cache, and if it's (IP) is not there or if the answer is expired, then it requests the root name server. The root name server (RNS) returns a list of .box TLD name servers, and since .box is uncommon, the RNS would not likely cache the IP of .box. But nonetheless, it then goes to the name server for .box and returns back to the DNS resolver to continue the search. Then, the DNS resolver again goes to the name server for .box, with a forward request to the .box, and then the authoritative nameserver is found for osmosis.box, which then returns the IP of osmosis.box. Then, the DNS resolver returns the IP address to the OS, and the OS returns the IP to the browser for the client to view.
  - To configure DNS and handle geolocation, we'll need to work with the .box domain registry and the SGX enclaves acting as authoritative name servers.
  - When a user requests osmosis.box, the browser sends a query to the DNS resolver. The resolver checks its cache and, if necessary, sends a query to the root name server. The root name server responds with the authoritative name servers for the .box TLD. The resolver then queries the .box authoritative name servers, which are SGX enclaves. The SGX enclaves, using a secure and verifiable process, determine the appropriate IP address for the user's location and return it to the resolver. The resolver caches the response and sends the IP address back to the user's browser. Configure the .box registry to recognize your SGX enclaves as the authoritative name servers for osmosis.box. Implement a secure and verifiable process within the SGX enclaves to handle DNS queries and determine the appropriate IP address based on the user's location. Ensure that the SGX enclaves can securely communicate with each other and maintain a consistent view of the DNS records.

### Challenge #5: Pre Geo Distribution
- **SGX and MPC Custody Coordination**:
  - Utilizing SGX (Software Guard Extensions) and MPC (Multi-Party Computation) for cryptographic key custody significantly enhances security by ensuring no single entity has access to the full cryptographic key, thus mitigating the risk of compromise. This approach is particularly effective in managing keys for DNSSEC (DNS Security Extensions), adding robust security layers to the domain name resolution process. The implementation of a Trusted Execution Environment (TEE) within SGX enclaves safeguards isolated yet verifiable IP resolution processes, preventing unauthorized access and ensuring that only verified, correctly functioning code can manipulate these keys.
  - Clarity on the MPC – Shamir's Secret Sharing to distribute the private key among multiple SGX enclaves. Each SGX enclave holds a share of the private key, and a threshold number of shares is going to be required to reconstruct the key. Then, when a DNS query or other operation requires the private key (e.g., transferring the osmosis.box domain), the Osmosis enclaves engage in an MPC protocol to jointly compute the operation without revealing their individual key shares. The MPC protocol ensures that no single SGX enclave has access to the complete private key. For that to work, we'd need to implement comm channels (e.g., TLS for Enclaves, such as EGo) between the SGX enclaves to protect the MPC protocol messages. And optionally, we could rotate the key shares and update the MPC configuration to maintain the security of the key shares, which should be determined by the DFE governance charter.
- **Code and IP Handling Protocols**:
  - The system's architecture delegates to a designated SGX enclave the sole responsibility of assigning IP addresses to other SGX enclaves that execute strictly verified front-end code. This crucial enclave in DNS resolution securely manages IP mappings by exclusively running authenticated and integrity-verified code. This stringent control mechanism not only preserves the integrity of IP allocations but also protects against unauthorized or malicious alterations. Distributing these responsibilities across a decentralized network of SGX enclaves reduces risks associated with centralized DNS management and significantly bolsters resilience against both internal and external threats. The integration of HTTPS encrypts data in transit, thereby fortifying the security framework of the entire system. Collectively, SGX's trusted execution capabilities, the decentralized architecture, and HTTPS's secure communication protocol create a formidable infrastructure for DNS management.
- **Decentralized Mapping of IP to SGX Enclaves**:
  - Our objective is to decentralize DNS processes and bolster their security. To this end, Cloudflare plays a pivotal role by ensuring that IP mappings are updated in real-time and by providing robust defenses against DDoS attacks. Concurrently, HTTPS is instrumental in securing network communications. It safeguards the integrity of queries sent to SGX for DNS resolution, ensuring that all communication channels are secure and immune to interception or tampering. This dual approach of utilizing Cloudflare for dynamic updates and resilience, combined with HTTPS for secure communication, significantly enhances the overall security and reliability of our DNS processes.
- **Pre Geo Distribution considerations p.1 - Getting around Cloudflare's gateways to enable user access but protect untrusted server feedback loops**:
  - Rate Limit:
    - We could use client-side storage (e.g., localStorage or IndexedDB) to store the user's request count & timestamp. If the user exceeds the rate limit, the frontend code can throttle or block further requests. Also, implementing an exponential backoff algorithm to gradually increase the waiting time between requests if a user repeatedly exceeds that limit.
  - Proof-of-Work thin wrapper:
    - Implement a client-side proof-of-work system, such as Hashcash, to prevent users from spamming requests. When a user initiates a request, generate a unique challenge (e.g., a random string) and set a difficulty level based on the current network load. Require the user's browser to solve the PoW puzzle by finding a solution that meets the difficulty criteria (e.g., a hash with a specific number of leading zeros). Once the user's browser solves the puzzle, attach the solution to the request and send it to the server. The server verifies the PoW solution before processing the request, discarding requests with invalid or missing solutions. Adjust the PoW difficulty dynamically based on the network load to maintain a balance between security and usability.
  - Real-Time Updates & Client-Side Caching:
    - Use the Cache API or IndexedDB to store frequently accessed data, such as token balances, exchange rates, or pool information. Before even sending a request, check the client-side cache for the required data. If available, use the cached data instead of making a network request. For that to work well, we'd need to implement cache invalidation strategies to ensure that the cached data remains up to date (e.g., time-based expiration, server-sent events, or websockets for real-time updates).
  - Carrier Aggregation for our mobile DFEs:
    - Instead of sending separate requests for token balances, exchange rates, and pool depths, combine them into a single request. We can include memoization tactics for NP routing problem sets.
- **Pre Geo Distribution considerations p.2 - Addressing HTTPS Enforcement in System Design**:
  - Background: HTTP Strict Transport Security (HSTS) is a policy mechanism designed to enhance web security by ensuring that connections between browsers and servers are conducted exclusively over HTTPS, thereby safeguarding against man-in-the-middle attacks.
  - Concern: The feasibility of initially enforcing HTTPS in our decentralized front-end design.
  - Resolution: Standard practice necessitates at least one successful HTTPS connection to set the HSTS header, instructing the browser to default to HTTPS for future requests. To enforce HTTPS from the onset, we incorporate a preloading step, adding the domain to the HSTS preload list integrated into browsers. This proactive measure ensures the browser defaults to HTTPS from the initial request, eliminating reliance on initial HTTP contact. Moreover, employing DNS-based security extensions like DNS-over-HTTPS (DoH) and DNS-over-TLS (DoT) encrypts DNS queries, reinforcing the security stance against the use of unsecured HTTP.
  - Clarity: DoH sends and receives DNS queries over HTTPS, which uses HTTP for communication but secures it with TLS. This encapsulation makes DNS queries indistinguishable from regular HTTPS traffic, allowing DoH to operate through standard HTTPS ports (typically port 443). This integration helps bypass network restrictions or censorship, as blocking DoH would require blocking all HTTPS traffic, which is impractical. In contrast, DoT secures DNS by encapsulating queries in TLS-encrypted connections, similar to HTTPS, but uses a separate protocol and typically operates over port 853. This provides the same levels of confidentiality, integrity, and authenticity as DoH but keeps DNS queries distinct and encrypted, simplifying network management and monitoring while being easier to filter or block. DoH is ideal for avoiding deep packet inspection or port-specific blocking, as it blends DNS queries with regular HTTPS traffic. Conversely, DoT is better for scenarios that require a distinct and secure channel for DNS queries.

### Challenge #6: DFE Governance
- **Objective**: Create a governance charter for the decentralized Osmosis frontend.
- **DFE Governance Principles**:
  - The civil war for privacy and verifiability, in the race for internet sovereignty and true identity.
  - Decentralization:
    - No single entity has control over the frontend governance.
    - Some voting mechanism for decision-making. 
    - Encouraging not just one, but possibly multiple networks to participate since Osmosis is unique in the way it uses various IBC-connected ecosystems.
  - Transparency:
    - Decisions and the prior process are made publicly accessible, likely using the same DFE infrastructure to host this. 
    - A record of votes, proposals, and outcomes.
  - Inclusivity:
    - An equal opportunity to govern with or without paid tokens.
  - Security:
    - I think a code audit / key rotation schedule is good, plus a 'things to avoid' section can be outlined here.
  - Accountability:
    - I think translating any slashing mechanism or conflict resolutions for when code goes rogue should be stated here. Code is controlled, but if there are systems that implement artificial automated updates to that code, we can discuss here how any implementations restrict or allow AI to manage protocol only if x, y, z implementations are done.
  - Collaboration:
    - This is really the voice of the charter where it's a call to action and states the importance of taking DFE to the public based on the key principles of the internet, freedom of code, equality, freedom of speech, and other ideas that are worth mentioning to promote this entire operation moving forward. Osmosis has a strong mission already, and I think this can be reinstated here as well.

## END

Repositories likely required to produce DFE demonstration:
1. Osmosis Front end (https://github.com/osmosis-labs/osmosis-frontend)
2. intel linux SGX SDK (https://github.com/intel/linux-sgx)
3. intel sgx driver (https://github.com/intel/linux-sgx-driver)
4. IBC-rs (https://github.com/cosmos/ibc-rs?tab=readme-ov-file) one or hermes (https://github.com/facebook/hermes) – icp A proxy for light client verification executed in TEE. (https://github.com/datachainlab/lcp) The goal is to have a working light client implementation of skip working in sgx to operate as a decentralized / confidential light client. 
5. purpose-built for secure TLS termination within SGX (https://github.com/lsds/TaLoS)
6. Gramine | Fortanix - Front end or Backend Code execution – for running code in the SGX, or likely performing code operations in the SGX that can be provided to the one hosting the DFE (https://github.com/gramineproject/gramine) For now we are attempting to provide a full implementation using the fortanix rust-sgx library, which provides multi-thread safety to improve performance and security for all of the programs we're currently testing in this library. 
7. running javascript in SGX (https://github.com/evervault/node-secureworker)
8. operations like random number generation, encryption, and hashing, or elliptic curve operations (https://github.com/openssl/openssl)
9. A homomorphic encryption library that can be used for secure MPC computations (https://github.com/microsoft/SEAL) We have been testing this library against other trusted sources such as the tool's Silence Labs will be providing us with to perform all of our MPC needs if found to be sufficent. Zama is looking to be a great contender as well. 
10. MPC Framework implementation (https://github.com/data61/MP-SPDZ)
11. BIND for DNS config and signing with DNSSEC (https://github.com/isc-projects/bind9)
12. For returning CA to SGX and have talos manage tls (https://github.com/certbot/certbot)