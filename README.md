# Rate Relay Service - Requirements Specification

## 1. Project Overview

### 1.1 Purpose
The Rate Relay service is a lightweight price data relay that fetches cryptocurrency price information from external sources and provides it to other services via HTTP API and WebSocket streaming. The initial version focuses on a minimal viable implementation with a single token pair (SOL/USDC) and single price source (Jupiter Price API v3).

### 1.2 Key Characteristics
- **Stateless**: Only maintains the most recent price data in memory
- **Real-time**: Provides current price information with 1-2 second refresh rate
- **Observable**: Prometheus metrics for monitoring and alerting

---

## 2. Functional Requirements

### 2.1 Price Data Fetching

- **Source**: Jupiter Price API v3
- **Token Pair**: SOL/USDC
- **Polling Interval**: 1-2 seconds (configurable)
- **Timeout**: 5 seconds per request
- **Error Handling**: Log errors but continue service operation
- **Validation**: Is timestamp recent and does price make sense?

### 2.2 HTTP API

- **Protocol**: HTTP/1.1
- **Format**: REST API with JSON responses
- **Port**: Configurable (default: 8080)
- **CORS**: Enabled for all origins (configurable)

### 2.3 WebSocket API
- Future feature


### 2.4 Prometheus Metrics

- **Path**: `/metrics`
- **Format**: Prometheus text exposition format
- **Port**: Same as HTTP API

Operators should configure alerts for:
- No successful price fetch in x+ seconds
- Price fetch error rate > 50% over 1 minute
- Price value unchanged for x+ minutes (potential stale data)


---

## 3. Technical Architecture

### 3.1 Technology Stack
- **Async Runtime**: Tokio
- **HTTP Server**: Axum
- **WebSocket**: tokio-tungstenite or Axum's WebSocket support
- **Metrics**: prometheus crate

### 3.2 Module Structure
https://miro.com/app/board/uXjVGWWp4cU=/?share_link_id=116254589137

---

## 4. Future Enhancements (Out of Scope for v1)

### 4.1 Multiple Price Sources
- Aggregate prices from multiple sources (Jupiter, Pyth, Binance)
- Fallback logic when primary source fails
- Price deviation alerts when sources disagree significantly

### 4.2 Multiple Token Pairs
- Configuration for multiple pairs (SOL/USDC, SOL/USDT, etc.)
- Dynamic pair addition without code changes
- Per-pair polling intervals

### 4.3 Streaming Price Data
- WebSocket connections to price sources (if available)
- Reduce latency by eliminating polling delay
- More efficient than polling

