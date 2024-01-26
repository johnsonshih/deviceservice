curl -d '{"Name":"Johnson"}' -H 'Content-Type: application/json' -X POST http://192.168.7.207:8080/api/v1/helloworld&&printf "\n"
curl -d '{"protocol":"onvif","data":{"id":"3fa1fe68-b915-4053-a3e1-ac15a21f5f91","properties":{}}}' \
  -H 'Content-Type: application/json' \
  -X POST http://192.168.1.145:8080/queryDeviceCredential \
  &&printf "\n"
curl -d '{"protocol":"onvif","data":{"reason":"add","device":{"id":"http://192.168.1.143:2020/onvif/device_service-3fa1fe68-b915-4053-a3e1-ac15a21f5f91","properties":{},"mounts":[],"device_specs":[]}}}'\
  -H 'Content-Type: application/json' \
  -X POST http://192.168.1.145:8080/deviceChange \
  &&printf "\n"


  curl -d '{"Name":"Johnson"}' -H 'Content-Type: application/json' -X GET http://192.168.1.145:30081/deviceservice/api/v1/helloworld&&printf "\n"
  curl -cert client.crt -key client.key -cacert ca.crt -d '{"Name":"Johnson"}' -H 'Content-Type: application/json' -X GET http://192.168.1.145:30081/deviceservice/api/v1/helloworld&&printf "\n"
  openssl s_client -connect 192.168.1.145:30081/deviceservice/api/v1/helloworld -cert client.crt -key client.key -CAfile ca.crt