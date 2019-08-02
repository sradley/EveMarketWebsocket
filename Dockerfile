FROM rust:1.31

WORKDIR /usr/src/eve_market_analysis
COPY . .

RUN cargo install --path .

CMD ["eve_market_analysis"]