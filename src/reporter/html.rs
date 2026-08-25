use crate::models::{D3GraphData, GraphLink, GraphNode, OsintReport};
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

pub fn build_graph_data(report: &OsintReport) -> D3GraphData {
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    let root_id = "target_phone".to_string();
    nodes.push(GraphNode {
        id: root_id.clone(),
        name: report.metadata.phone.clone(),
        group: "phone".to_string(),
        val: 25,
    });

    if let Some(ref nv) = report.results.numverify {
        if let Some(ref c) = nv.carrier {
            let cid = format!("carrier_{}", c);
            nodes.push(GraphNode {
                id: cid.clone(),
                name: format!("Carrier: {}", c),
                group: "carrier".to_string(),
                val: 14,
            });
            links.push(GraphLink {
                source: root_id.clone(),
                target: cid,
                label: "TELECOM".to_string(),
            });
        }
    }

    if let Some(ref _nl) = report.results.numlookup {
        let nlid = "numlookup_portal".to_string();
        nodes.push(GraphNode {
            id: nlid.clone(),
            name: "NumLookup Directory".to_string(),
            group: "carrier".to_string(),
            val: 12,
        });
        links.push(GraphLink {
            source: root_id.clone(),
            target: nlid,
            label: "REVERSE_DIRECTORY".to_string(),
        });
    }

    if let Some(ref hr) = report.results.hudsonrock {
        for (i, stealer) in hr.stealers.iter().enumerate() {
            let family = stealer.stealer_family.as_deref().unwrap_or("Malware");
            let sid = format!("stealer_{}_{}", family, i);
            nodes.push(GraphNode {
                id: sid.clone(),
                name: format!("Infection: {}", family),
                group: "stealer".to_string(),
                val: 18,
            });
            links.push(GraphLink {
                source: root_id.clone(),
                target: sid.clone(),
                label: "COMPROMISED_BY".to_string(),
            });

            for (j, login) in stealer.top_logins.iter().take(4).enumerate() {
                let lid = format!("login_{}_{}_{}", family, i, j);
                nodes.push(GraphNode {
                    id: lid.clone(),
                    name: login.clone(),
                    group: "account".to_string(),
                    val: 10,
                });
                links.push(GraphLink {
                    source: sid.clone(),
                    target: lid,
                    label: "LEAKED_CREDENTIAL".to_string(),
                });
            }
        }
    }

    for (i, repo) in report.results.github.iter().take(4).enumerate() {
        let gid = format!("github_{}", i);
        nodes.push(GraphNode {
            id: gid.clone(),
            name: repo.repository.clone(),
            group: "repo".to_string(),
            val: 12,
        });
        links.push(GraphLink {
            source: root_id.clone(),
            target: gid,
            label: "CODE_LEAK".to_string(),
        });
    }

    for (i, post) in report.results.reddit.iter().take(3).enumerate() {
        let rid = format!("reddit_{}", i);
        nodes.push(GraphNode {
            id: rid.clone(),
            name: format!("r/{}: {}", post.subreddit, post.title.chars().take(25).collect::<String>()),
            group: "social".to_string(),
            val: 12,
        });
        links.push(GraphLink {
            source: root_id.clone(),
            target: rid,
            label: "MENTION".to_string(),
        });
    }

    for (i, dork) in report.results.dorks.iter().take(3).enumerate() {
        let did = format!("dork_{}", i);
        nodes.push(GraphNode {
            id: did.clone(),
            name: format!("Dork: {}", dork.category),
            group: "dork".to_string(),
            val: 12,
        });
        links.push(GraphLink {
            source: root_id.clone(),
            target: did,
            label: "SEARCH_FINDING".to_string(),
        });
    }

    D3GraphData { nodes, links }
}

const HTML_TEMPLATE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>SearchPhone OSINT Intelligence Graph - __PHONE__</title>
  <script src="https://d3js.org/d3.v7.min.js"></script>
  <style>
    :root {
      --bg: #0b0f19;
      --card-bg: #131b2e;
      --card-border: #1e293b;
      --text: #f1f5f9;
      --text-muted: #94a3b8;
      --accent: #38bdf8;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      line-height: 1.5;
      padding: 24px;
    }
    .container { max-width: 1280px; margin: 0 auto; }
    header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding-bottom: 20px;
      border-bottom: 1px solid var(--card-border);
      margin-bottom: 24px;
    }
    .brand { font-size: 24px; font-weight: 800; color: var(--accent); letter-spacing: -0.5px; }
    .badge {
      display: inline-block;
      padding: 4px 12px;
      border-radius: 9999px;
      font-size: 12px;
      font-weight: 700;
      text-transform: uppercase;
    }
    .grid { display: grid; grid-template-columns: 1fr 2fr; gap: 20px; margin-bottom: 24px; }
    @media (max-width: 900px) { .grid { grid-template-columns: 1fr; } }
    .card {
      background: var(--card-bg);
      border: 1px solid var(--card-border);
      border-radius: 12px;
      padding: 20px;
    }
    .card h3 {
      font-size: 14px;
      font-weight: 700;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      margin-bottom: 14px;
    }
    .profile-item { display: flex; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #1a233a; font-size: 14px; }
    .profile-item:last-child { border-bottom: none; }
    .profile-label { color: var(--text-muted); }
    .profile-val { font-weight: 600; color: #fff; }
    #graph-container {
      width: 100%;
      height: 520px;
      background: #070a12;
      border-radius: 8px;
      position: relative;
      overflow: hidden;
    }
    svg { width: 100%; height: 100%; }
    .legend {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
      margin-top: 12px;
      font-size: 12px;
      color: var(--text-muted);
    }
    .legend-item { display: flex; align-items: center; gap: 6px; }
    .legend-dot { width: 10px; height: 10px; border-radius: 50%; }
    pre {
      background: #070a12;
      padding: 16px;
      border-radius: 8px;
      overflow-x: auto;
      font-size: 12px;
      color: #94a3b8;
    }
  </style>
</head>
<body>
  <div class="container">
    <header>
      <div>
        <div class="brand">🕵 SearchPhone OSINT Intelligence</div>
        <div style="font-size: 13px; color: var(--text-muted); margin-top: 4px;">Target Investigation Report & Entity Relationship Graph</div>
      </div>
      <div style="text-align: right;">
        <span class="badge" style="background: __LEVEL_COLOR__22; color: __LEVEL_COLOR__; border: 1px solid __LEVEL_COLOR__;">
          __THREAT_LEVEL__ THREAT (Score: __THREAT_SCORE__/100)
        </span>
        <div style="font-size: 12px; color: var(--text-muted); margin-top: 6px;">__TIMESTAMP__</div>
      </div>
    </header>

    <div class="grid">
      <div class="card">
        <h3>Target Profile</h3>
        <div class="profile-item"><span class="profile-label">Phone</span><span class="profile-val">__PHONE__</span></div>
        <div class="profile-item"><span class="profile-label">Region</span><span class="profile-val">__REGION__</span></div>
        <div class="profile-item"><span class="profile-label">Risk Assessment</span><span class="profile-val" style="color: __LEVEL_COLOR__">__THREAT_LEVEL__</span></div>
        <div class="profile-item"><span class="profile-label">Infostealers Found</span><span class="profile-val">__STEALERS_COUNT__</span></div>
        <div class="profile-item"><span class="profile-label">GitHub Code Leaks</span><span class="profile-val">__GITHUB_COUNT__</span></div>
        <div class="profile-item"><span class="profile-label">Social Discussions</span><span class="profile-val">__REDDIT_COUNT__</span></div>
      </div>

      <div class="card">
        <h3>Entity Relationship Graph (Interactive)</h3>
        <div id="graph-container"></div>
        <div class="legend">
          <div class="legend-item"><span class="legend-dot" style="background: #38bdf8"></span> Target Phone</div>
          <div class="legend-item"><span class="legend-dot" style="background: #ef4444"></span> Malware Stealer</div>
          <div class="legend-item"><span class="legend-dot" style="background: #f59e0b"></span> Exposed Account</div>
          <div class="legend-item"><span class="legend-dot" style="background: #a855f7"></span> Git Repository</div>
          <div class="legend-item"><span class="legend-dot" style="background: #ec4899"></span> Social Post</div>
          <div class="legend-item"><span class="legend-dot" style="background: #10b981"></span> Carrier / Telecom</div>
        </div>
      </div>
    </div>

    <div class="card">
      <h3>Full OSINT Intelligence JSON Payload</h3>
      <pre><code>__RAW_JSON__</code></pre>
    </div>
  </div>

  <script>
    const data = __GRAPH_JSON__;
    const container = document.getElementById('graph-container');
    const width = container.clientWidth || 700;
    const height = container.clientHeight || 520;

    const svg = d3.select("#graph-container")
      .append("svg")
      .attr("viewBox", [0, 0, width, height]);

    const colorMap = {
      phone: "#38bdf8",
      stealer: "#ef4444",
      account: "#f59e0b",
      repo: "#a855f7",
      social: "#ec4899",
      dork: "#06b6d4",
      carrier: "#10b981"
    };

    const simulation = d3.forceSimulation(data.nodes)
      .force("link", d3.forceLink(data.links).id(d => d.id).distance(90))
      .force("charge", d3.forceManyBody().strength(-240))
      .force("center", d3.forceCenter(width / 2, height / 2));

    const g = svg.append("g");

    svg.call(d3.zoom().scaleExtent([0.4, 4]).on("zoom", (event) => {
      g.attr("transform", event.transform);
    }));

    const link = g.append("g")
      .selectAll("line")
      .data(data.links)
      .join("line")
      .attr("stroke", "#334155")
      .attr("stroke-width", 1.5)
      .attr("stroke-opacity", 0.6);

    const node = g.append("g")
      .selectAll("g")
      .data(data.nodes)
      .join("g")
      .call(d3.drag()
        .on("start", (event, d) => {
          if (!event.active) simulation.alphaTarget(0.3).restart();
          d.fx = d.x; d.fy = d.y;
        })
        .on("drag", (event, d) => { d.fx = event.x; d.fy = event.y; })
        .on("end", (event, d) => {
          if (!event.active) simulation.alphaTarget(0);
          d.fx = null; d.fy = null;
        }));

    node.append("circle")
      .attr("r", d => d.val || 12)
      .attr("fill", d => colorMap[d.group] || "#64748b")
      .attr("stroke", "#fff")
      .attr("stroke-width", 1.5);

    node.append("text")
      .attr("x", 14)
      .attr("y", 4)
      .text(d => d.name)
      .attr("fill", "#f8fafc")
      .attr("font-size", "11px")
      .attr("font-family", "sans-serif")
      .attr("paint-order", "stroke")
      .attr("stroke", "#070a12")
      .attr("stroke-width", 3);

    simulation.on("tick", () => {
      link
        .attr("x1", d => d.source.x)
        .attr("y1", d => d.source.y)
        .attr("x2", d => d.target.x)
        .attr("y2", d => d.target.y);

      node.attr("transform", d => `translate(${d.x},${d.y})`);
    });
  </script>
</body>
</html>"##;

pub fn generate_html_report(report: &OsintReport) -> String {
    let graph_data = build_graph_data(report);
    let graph_json = serde_json::to_string(&graph_data).unwrap_or_else(|_| "{}".to_string());
    let raw_report_json = serde_json::to_string_pretty(report).unwrap_or_default();

    let threat_score = report
        .results
        .threat_reputation
        .as_ref()
        .map(|t| t.risk_score)
        .unwrap_or(0);
    let threat_level = report
        .results
        .threat_reputation
        .as_ref()
        .map(|t| t.risk_level.as_str())
        .unwrap_or("CLEAN");

    let level_color = match threat_level {
        "CRITICAL" | "HIGH" => "#ef4444",
        "MEDIUM" => "#f59e0b",
        "LOW" => "#3b82f6",
        _ => "#10b981",
    };

    let stealers_count = report.results.hudsonrock.as_ref().map(|hr| hr.stealers.len()).unwrap_or(0);

    HTML_TEMPLATE
        .replace("__PHONE__", &report.metadata.phone)
        .replace("__REGION__", &report.metadata.region.to_uppercase())
        .replace("__TIMESTAMP__", &report.metadata.timestamp)
        .replace("__THREAT_LEVEL__", threat_level)
        .replace("__THREAT_SCORE__", &threat_score.to_string())
        .replace("__LEVEL_COLOR__", level_color)
        .replace("__STEALERS_COUNT__", &stealers_count.to_string())
        .replace("__GITHUB_COUNT__", &report.results.github.len().to_string())
        .replace("__REDDIT_COUNT__", &report.results.reddit.len().to_string())
        .replace("__GRAPH_JSON__", &graph_json)
        .replace("__RAW_JSON__", &raw_report_json)
}

pub fn export_html(report: &OsintReport, report_dir: &str, filename: &str) -> Result<String, String> {
    create_dir_all(report_dir).map_err(|e| e.to_string())?;
    let path = Path::new(report_dir).join(filename);

    let html_content = generate_html_report(report);
    let mut file = File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(html_content.as_bytes()).map_err(|e| e.to_string())?;

    Ok(path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OsintResults, ReportMetadata};

    #[test]
    fn test_generate_html_report() {
        let report = OsintReport {
            metadata: ReportMetadata {
                phone: "+16502530000".to_string(),
                region: "US".to_string(),
                timestamp: "2026-08-25T00:00:00Z".to_string(),
                tool: "SearchPhone OSINT".to_string(),
            },
            results: OsintResults::default(),
        };

        let html = generate_html_report(&report);
        assert!(html.contains("SearchPhone OSINT Intelligence"));
        assert!(html.contains("+16502530000"));
    }
}
