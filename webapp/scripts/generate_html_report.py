#!/usr/bin/env python3
"""Generate a standalone HTML validation report (no dependencies required)."""

import json
import sys
from datetime import datetime
from pathlib import Path


def load_test_results():
    """Load test results from JSON file."""
    results_file = Path(__file__).parent.parent / "test_results.json"
    if not results_file.exists():
        print("❌ test_results.json not found. Run export_test_results.py first.")
        sys.exit(1)

    with open(results_file, "r") as f:
        return json.load(f)


def generate_html(report):
    """Generate HTML report."""
    summary = report["summary"]
    modules = report["modules"]
    examples = report["examples"]
    kpis = report["kpis"]

    html = f"""<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CalculiX Rust Solver Validation Report</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: #2c3e50;
            line-height: 1.6;
            padding: 20px;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            background: white;
            border-radius: 15px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.3);
            overflow: hidden;
        }}
        header {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 40px;
            text-align: center;
        }}
        header h1 {{
            font-size: 2.5em;
            margin-bottom: 10px;
        }}
        .subtitle {{
            font-size: 1.2em;
            opacity: 0.9;
        }}
        .content {{
            padding: 40px;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin-bottom: 40px;
        }}
        .stat-card {{
            background: #f8f9fa;
            padding: 25px;
            border-radius: 10px;
            text-align: center;
            border: 2px solid #e9ecef;
            transition: transform 0.3s;
        }}
        .stat-card:hover {{
            transform: translateY(-5px);
            box-shadow: 0 5px 15px rgba(0,0,0,0.1);
        }}
        .stat-value {{
            font-size: 2.5em;
            font-weight: bold;
            color: #667eea;
            margin: 10px 0;
        }}
        .stat-label {{
            color: #6c757d;
            text-transform: uppercase;
            font-size: 0.85em;
            letter-spacing: 1px;
        }}
        .section {{
            margin-bottom: 40px;
        }}
        .section h2 {{
            color: #2c3e50;
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 3px solid #667eea;
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            background: white;
            border-radius: 8px;
            overflow: hidden;
        }}
        th {{
            background: #667eea;
            color: white;
            padding: 15px;
            text-align: left;
            font-weight: 600;
        }}
        td {{
            padding: 15px;
            border-bottom: 1px solid #e9ecef;
        }}
        tr:hover {{
            background: #f8f9fa;
        }}
        .progress-bar {{
            width: 100%;
            height: 30px;
            background: #e9ecef;
            border-radius: 15px;
            overflow: hidden;
            position: relative;
        }}
        .progress-fill {{
            height: 100%;
            background: linear-gradient(90deg, #667eea 0%, #764ba2 100%);
            display: flex;
            align-items: center;
            justify-content: center;
            color: white;
            font-weight: bold;
            transition: width 0.5s ease;
        }}
        .badge {{
            display: inline-block;
            padding: 5px 12px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: 600;
        }}
        .badge-success {{
            background: #d4edda;
            color: #155724;
        }}
        .badge-info {{
            background: #d1ecf1;
            color: #0c5460;
        }}
        code {{
            background: #f4f4f4;
            padding: 3px 8px;
            border-radius: 4px;
            font-family: 'Courier New', monospace;
            color: #e83e8c;
        }}
        footer {{
            background: #f8f9fa;
            padding: 30px;
            text-align: center;
            color: #6c757d;
        }}
        .timestamp {{
            color: #95a5a6;
            font-size: 0.9em;
        }}
        .status-pass {{
            color: #28a745;
            font-weight: bold;
        }}
        .metric-value {{
            font-family: 'Courier New', monospace;
            color: #495057;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>🦀 CalculiX Rust Solver</h1>
            <div class="subtitle">Validation Report</div>
            <div class="timestamp">Generated: {datetime.utcnow().strftime('%Y-%m-%d %H:%M:%S')} UTC</div>
        </header>

        <div class="content">
            <!-- Summary Statistics -->
            <div class="stats-grid">
                <div class="stat-card">
                    <div class="stat-label">Total Tests</div>
                    <div class="stat-value">{summary['total_tests']}</div>
                    <span class="badge badge-success">{summary['passing_tests']} passing</span>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Pass Rate</div>
                    <div class="stat-value">{summary['pass_rate']:.1f}%</div>
                    <div class="progress-bar">
                        <div class="progress-fill" style="width: {summary['pass_rate']}%">{summary['pass_rate']:.0f}%</div>
                    </div>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Lines of Code</div>
                    <div class="stat-value">{report['lines_of_code']:,}</div>
                    <span class="badge badge-info">Rust LOC</span>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Element Types</div>
                    <div class="stat-value">{kpis['num_element_types']}</div>
                    <code>{', '.join(kpis['supported_elements'])}</code>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Test Modules</div>
                    <div class="stat-value">{len(modules)}</div>
                    <span class="badge badge-success">Active</span>
                </div>
                <div class="stat-card">
                    <div class="stat-label">Avg Test Time</div>
                    <div class="stat-value">{kpis['avg_test_time_ms']:.1f}</div>
                    <span class="badge badge-info">milliseconds</span>
                </div>
            </div>

            <!-- Test Modules -->
            <div class="section">
                <h2>🧪 Test Modules</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Module</th>
                            <th>Description</th>
                            <th>Tests</th>
                            <th>Passing</th>
                            <th>Pass Rate</th>
                        </tr>
                    </thead>
                    <tbody>
"""

    for module in modules:
        pass_rate = (module["passing"] / module["tests"] * 100) if module["tests"] > 0 else 0
        html += f"""
                        <tr>
                            <td><strong>{module['name']}</strong></td>
                            <td>{module['description']}</td>
                            <td>{module['tests']}</td>
                            <td class="status-pass">{module['passing']}</td>
                            <td>
                                <div class="progress-bar">
                                    <div class="progress-fill" style="width: {pass_rate}%">{pass_rate:.0f}%</div>
                                </div>
                            </td>
                        </tr>
"""

    html += """
                    </tbody>
                </table>
            </div>

            <!-- Example Problems -->
            <div class="section">
                <h2>📝 Example Problems</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Example</th>
                            <th>Element Type</th>
                            <th>Nodes</th>
                            <th>Elements</th>
                            <th>DOFs</th>
                            <th>Validations</th>
                        </tr>
                    </thead>
                    <tbody>
"""

    for example in examples:
        html += f"""
                        <tr>
                            <td><strong>{example['name']}</strong></td>
                            <td><code>{example['element_type']}</code></td>
                            <td>{example['num_nodes']}</td>
                            <td>{example['num_elements']}</td>
                            <td>{example['num_dofs']}</td>
                            <td class="status-pass">{len(example['validations'])} ✓</td>
                        </tr>
"""

    html += """
                    </tbody>
                </table>
            </div>

            <!-- Validation Results -->
            <div class="section">
                <h2>✅ Validation Results</h2>
                <table>
                    <thead>
                        <tr>
                            <th>Example</th>
                            <th>Metric</th>
                            <th>Computed</th>
                            <th>Analytical</th>
                            <th>Error</th>
                            <th>Status</th>
                        </tr>
                    </thead>
                    <tbody>
"""

    for example in examples:
        for validation in example["validations"]:
            error_str = (
                f"{validation['error'] * 100:.4f}%"
                if validation["error"] is not None
                else "N/A"
            )
            analytical_str = (
                f"{validation['analytical']:.6f}"
                if validation["analytical"] is not None
                else "N/A"
            )
            html += f"""
                        <tr>
                            <td><strong>{example['name']}</strong></td>
                            <td>{validation['metric']}</td>
                            <td class="metric-value">{validation['computed']:.6f}</td>
                            <td class="metric-value">{analytical_str}</td>
                            <td>{error_str}</td>
                            <td class="status-pass">{'✓ PASS' if validation['passed'] else '✗ FAIL'}</td>
                        </tr>
"""

    html += f"""
                    </tbody>
                </table>
            </div>
        </div>

        <footer>
            <p><strong>CalculiX Rust Solver Validation System</strong></p>
            <p class="timestamp">Git commit: {report['git_commit']}</p>
            <p class="timestamp">Generated by webapp validation module v0.1.0</p>
        </footer>
    </div>
</body>
</html>
"""

    return html


def main():
    """Main function."""
    print("📊 Generating HTML validation report...")

    # Load test results
    report = load_test_results()

    # Generate HTML
    html = generate_html(report)

    # Save to file
    output_file = Path(__file__).parent.parent / "validation_report.html"
    with open(output_file, "w") as f:
        f.write(html)

    print(f"✅ Report generated: {output_file}")
    print(f"\n   Open in browser: file://{output_file.absolute()}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
