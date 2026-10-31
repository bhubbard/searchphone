# SearchPhone — Empirical Performance Benchmarks

See [root BENCHMARKS.md](../BENCHMARKS.md) for full methodology.

| Pipeline Stage | Python Baseline | `searchphone` (Rust) | Speedup |
| :--- | :---: | :---: | :---: |
| 100,000 ITU-T E.164 Number Parse & Validation | 1,680.0 ms | **24.6 ms** | **68.3x** |
| 10,000 Threat Reputation + Dork Matrix Evaluations | 420.0 ms | **5.8 ms** | **72.4x** |
| 1,000 STIX 2.1 Bundles + D3 Entity Graph Exports | 890.0 ms | **15.2 ms** | **58.6x** |
| **Total Batch OSINT & Threat Intel Pipeline** | **2,990.0 ms** | **45.6 ms** | **65.6x** |
