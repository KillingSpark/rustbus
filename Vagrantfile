# -*- mode: ruby -*-
# vi: set ft=ruby :
Vagrant.configure("2") do |config|
  config.vm.box = "centos/stream8"
  config.vm.box_version = "20210210.0"
  config.vm.provision "shell", inline: <<-EOF
yum -y update
yum -y install curl
EOF
  # install rust for the vagrant user
  config.vm.provision "shell",  privileged: false, inline: <<-EOF
curl https://sh.rustup.rs -sSf | sh -s -- -y;
  EOF

  # enable guest folder in vbox
  # run 'vagrant plugin install vagrant-vbguest' to enjoy it
  config.vm.provider "virtualbox" do |vb|
    config.vm.synced_folder ".", "/vagrant", type: "virtualbox"
  end
end
