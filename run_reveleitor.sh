NUM=$1
WEBCASTCODE=$2
SEDE=$3
SECRET=$4
PORT=$(expr 10000 + $NUM)
URL="https://global.naquadah.com.br/bocaBZ2222fe/admin/report/webcast.php?webcastcode=${WEBCASTCODE}"
#HOSTNAME="animeitor.naquadah.com.br"
HOSTNAME="localhost"
SECRET=$(echo $RANDOM | md5sum | head -c 5)

echo "$SEDE => http://${HOSTNAME}:${PORT}/reveleitor.html?secret=${SECRET}"

./target/release/simples ${URL} --port ${PORT} --config config/Regional_2022.toml --secret ${SECRET} 2>&1 > logs/reveleitor_${NUM}.txt
