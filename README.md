First, make sure you're in Linux environment because we need to grant network admin capabilities to our compiled binary (CAP_NET_ADMIN).

Second, run the run script (run.sh) using these commands:
chmod +x ./run.sh
./run.sh

After entering your password, you can open another terminal and try using netcat to send and receive packet from the ip in the run.sh script (192.168.0.1)

nc 192.168.0.2 443 (tcp)
Or
nc 192.168.0.3 80 (http)
or
sudo tshark -i tun0





