use neo4rs::*;
use std::collections::BTreeSet;
use std::time::Instant;

#[derive(Debug, serde::Deserialize)]
struct Edge {
    from: String,
    to: String,
    ex: String,
    invert: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "127.0.0.1:7687";
    let g = Graph::new(url, "neo4j", "password").await?;

    // まず全クリア
    g.run(query("Match ()-[e]-() DELETE e")).await?;
    g.run(query("Match (n) DELETE n")).await?;

    let edges = [
        ("ETH", "Binance", "USD"),
        ("ETH", "Kraken", "USD"),
        ("USD", "Exchange", "JPY"),
        ("ETH", "Bitbank", "JPY"),
        ("BTC", "Bitbank", "JPY"),
    ];

    let mut nodes = BTreeSet::new();
    for (from, ex, to) in edges {
        nodes.insert(from);
        nodes.insert(to);
    }

    for node in nodes {
        g.run(query(("CREATE (n: Token { name: $name })")).param("name", node))
            .await?;
    }

    for (from, ex, to) in edges {
        let q = query(
            "\
        MATCH (n1: Token { name: $from }), (n2: Token { name: $to }) \
        CREATE (n1)-[:Conversion { from: $from, to: $to, ex: $ex, invert: false }]->(n2)
        ",
        )
        .param("from", from)
        .param("to", to)
        .param("ex", ex);
        g.run(q).await?;

        // 逆エッジもいれる。
        let q = query(
            "\
        MATCH (n1: Token { name: $from }), (n2: Token { name: $to }) \
        CREATE (n1)-[:Conversion { from: $from, to: $to, ex: $ex, invert: true }]->(n2)
        ",
        )
        .param("from", to)
        .param("to", from)
        .param("ex", ex);
        g.run(q).await?;
    }

    let t = Instant::now();
    let q = query(
        "Match (n1: Token { name: $from })-[p:Conversion*1..3]->(n2: Token { name: $to }) RETURN p",
    )
    .param("from", "ETH")
    .param("to", "JPY");
    let mut st = g.execute(q).await?;
    while let Ok(Some(row)) = st.next().await {
        let es: Vec<Edge> = row.to()?;
        dbg!(&es);
    }
    dbg!(t.elapsed());

    Ok(())
}
