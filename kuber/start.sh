minikube start
kubectl apply -f mongo_cfg.yaml
while [[ $(kubectl get pods -l app=mongo -o 'jsonpath={..status.conditions[?(@.type=="Ready")].status}') != "True" ]]; do echo "waiting for pod mongo" && sleep 1; done
kubectl apply -f ./postgres_cfg.yaml
while [[ $(kubectl get pods -l app=postgres -o 'jsonpath={..status.conditions[?(@.type=="Ready")].status}') != "True" ]]; do echo "waiting for pod postges" && sleep 1; done
kubectl apply -f ./myapp_cfg.yaml
while [[ $(kubectl get pods -l app=myapp -o 'jsonpath={..status.conditions[?(@.type=="Ready")].status}') != "True" ]]; do echo "waiting for pod myapp" && sleep 1; done
export POD_NAME=$(kubectl get pods --selector=app=myapp --template '{{range .items}}{{.metadata.name}}{{"\n"}}{{end}}')
kubectl port-forward pod/$POD_NAME 3030:3030