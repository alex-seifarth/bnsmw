FROM ubuntu:24.04 AS build
RUN <<EOF
apt-get update
apt-get install -y build-essential libboost-all-dev git cmake zlib1g-dev libdbus-glib-1-dev

git clone https://github.com/COVESA/dlt-daemon.git
cd dlt-daemon
git fetch --all --tags --prune
git checkout tags/v2.18.10
mkdir build
cmake -DCMAKE_BUILD_TYPE=Release -B build -S .
cmake --build build --target install --config Release
tar cfv pkg_dlt.tar -T build/install_manifest.txt
cd ..

git clone https://github.com/COVESA/vsomeip.git
cd vsomeip
git fetch --all --tags --prune
git checkout tags/3.5.1
mkdir build
cmake -DCMAKE_BUILD_TYPE=Release -B build -S .
cmake --build build --target install --config Release
tar cfv pkg_vsomeip.tar -T build/install_manifest.txt
cd ..

tar cvf /tmp/boost.tar /usr/include/boost/

EOF

FROM ubuntu:24.04 AS final
COPY --from=build /dlt-daemon/pkg_dlt.tar /vsomeip/pkg_vsomeip.tar /tmp/boost.tar /tmp/
RUN <<EOF
apt-get update
apt-get install -y libboost-filesystem-dev libboost-thread-dev curl

tar xvf /tmp/pkg_dlt.tar
rm /tmp/pkg_dlt.tar
tar xvf /tmp/pkg_vsomeip.tar
rm /tmp/pkg_vsomeip.tar
tar xvf /tmp/boost.tar
rm /tmp/boost.tar

ldconfig
EOF
