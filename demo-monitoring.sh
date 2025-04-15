#!/bin/bash
# QitOps Agent Monitoring Demo

echo "QitOps Agent Monitoring Demo"
echo "==========================="
echo

echo "Step 1: Starting monitoring stack..."
./start-monitoring.sh
echo

echo "Step 2: Pushing test metrics..."
chmod +x push-test-metrics.sh
./push-test-metrics.sh
echo

echo "Step 3: Opening Grafana dashboard..."
if command -v xdg-open &> /dev/null; then
    xdg-open http://localhost:3000/d/qitops-dashboard/qitops-dashboard
elif command -v open &> /dev/null; then
    open http://localhost:3000/d/qitops-dashboard/qitops-dashboard
else
    echo "Please open http://localhost:3000/d/qitops-dashboard/qitops-dashboard in your browser"
fi
echo

echo "Demo is running!"
echo
echo "- Grafana: http://localhost:3000 (admin/qitops)"
echo "- Prometheus: http://localhost:9090"
echo "- Push Gateway: http://localhost:9091"
echo
echo "Press Enter to stop the demo and clean up..."
read

echo
echo "Stopping monitoring stack..."
./stop-monitoring.sh
echo

echo "Demo completed!"
echo
