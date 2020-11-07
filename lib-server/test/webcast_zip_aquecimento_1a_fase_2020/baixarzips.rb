

url = "https://global.naquadah.com.br/bocasecret/admin/report/webcast.php?webcastcode=FGHsalIJKBrasil"
(1..100000).each do |x|
  puts("dormindo...")
  sleep 2
  puts("baixando...")
  system("wget #{url} -O arquivo_#{x}.zip")
end

