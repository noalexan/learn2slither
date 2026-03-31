#!/usr/bin/python3

import re
import os
import sys
import threading
from dash import Dash, dcc, html, Input, Output
import plotly.graph_objects as go
import webbrowser

ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')

def parse_gen_line(line):
    pattern = r'Generation\s+(\d+)/(\d+)\s+=>\s+min:\s+(\d+),\s+max:\s+(\d+),\s+avrg:\s+(\d+),\s+len\s*\(max\):\s+(\d+)'
    match = re.match(pattern, line.strip())
    if match:
        return {
            'generation': int(match.group(1)),
            'total': int(match.group(2)),
            'min': int(match.group(3)),
            'max': int(match.group(4)),
            'avg': int(match.group(5)),
            'len_max': int(match.group(6))
        }
    return None

class LivePlotDash:
    def __init__(self, path):
        self.path = path
        self.gens = []
        self.mins = []
        self.maxs = []
        self.avgs = []
        self.file_size = 0
        self.total_gens = 0  # ✅ Fixed: class attribute
        
        self.app = Dash(__name__)
        self.app.layout = html.Div([
            html.H1(id='title', style={'textAlign': 'center'}),
            dcc.Graph(id='live-graph'),
            dcc.Interval(id='interval', interval=1000, n_intervals=0),
            html.Div(id='status', style={'padding': '10px', 'fontSize': 14})
        ])
        
        self.app.callback(
            [Output('live-graph', 'figure'), Output('status', 'children')],
            Input('interval', 'n_intervals')
        )(self.update_graph)
        
    def update_data(self):
        if not os.path.exists(self.path):
            return False
            
        current_size = os.path.getsize(self.path)
        if current_size <= self.file_size:
            return False
            
        with open(self.path, 'r') as file:
            file.seek(self.file_size)
            new_lines = file.readlines()
            self.file_size = current_size
            
        new_datas = []
        for line in new_lines:
            data = parse_gen_line(ansi_escape.sub('', line))
            if data:
                new_datas.append(data)
        
        for data in new_datas:
            self.gens.append(data['generation'])
            self.mins.append(data['min'])
            self.maxs.append(data['max'])
            self.avgs.append(data['avg'])
            self.total_gens = data['total']  # Update total
        
        return bool(new_datas)
    
    def update_graph(self, n):
        updated = self.update_data()
        
        status = f"📁 File: {self.file_size:,} bytes "
        if self.gens:
            status += f"| 🐍 Gen {self.gens[-1]}/{self.total_gens} "
            status += f"| 📈 Min:{self.mins[-1]:,} Max:{self.maxs[-1]:,} Avg:{self.avgs[-1]:,}"
            status += f" | 📊 {len(self.gens)} points"
        else:
            status += "| ⏳ Waiting for GA output..."
        
        fig = go.Figure()
        if self.gens:
            fig.add_trace(go.Scatter(
                x=self.gens, y=self.mins, mode='lines+markers', 
                name='Min Fitness', line=dict(color='blue', width=2),
                marker=dict(size=4)
            ))
            fig.add_trace(go.Scatter(
                x=self.gens, y=self.maxs, mode='lines+markers', 
                name='Max Fitness', line=dict(color='red', width=2),
                marker=dict(size=4, color='red')
            ))
            fig.add_trace(go.Scatter(
                x=self.gens, y=self.avgs, mode='lines+markers', 
                name='Avg Fitness', line=dict(color='green', width=3),
                marker=dict(size=6, color='green')
            ))
            
            fig.update_layout(
                title=f"🐍 Snake GA Evolution - LIVE ({len(self.gens)} generations)",
                xaxis_title="Generation", yaxis_title="Fitness Score",
                hovermode='x unified', showlegend=True,
                width=1200, height=700,
                template='plotly_white'
            )
        
        return fig, status
    
    def run(self):
        print("🚀 Snake GA LIVE DASH PLOT")
        print(f"📁 Watching: {self.path}")
        print("🌐 Browser opens in 2 seconds... Ctrl+C to stop")
        
        def open_browser():
            webbrowser.open('http://127.0.0.1:8050/')
        
        threading.Timer(2.0, open_browser).start()
        self.app.run(debug=False, host='127.0.0.1', port=8050, use_reloader=False)

if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "output.txt"
    
    import subprocess
    try:
        import plotly
    except ImportError:
        print("📦 Installing plotly + dash...")
        subprocess.check_call([sys.executable, "-m", "pip", "install", "plotly", "dash"])
        import plotly
    
    plotter = LivePlotDash(path)
    plotter.run()

