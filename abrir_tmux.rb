#!/bin/ruby

sedes = %w{FGHsalIJKGlobal
FGHsalIJKBrasil
FGHsalIJKac
FGHsalIJKal
FGHsalIJKam
FGHsalIJKap
FGHsalIJKba
FGHsalIJKce
FGHsalIJKdf
FGHsalIJKes
FGHsalIJKgo
FGHsalIJKma
FGHsalIJKmg
FGHsalIJKms
FGHsalIJKmt
FGHsalIJKpa
FGHsalIJKpb
FGHsalIJKpe
FGHsalIJKpi
FGHsalIJKpr
FGHsalIJKrj
FGHsalIJKrn
FGHsalIJKro
FGHsalIJKrr
FGHsalIJKrs
FGHsalIJKsc
FGHsalIJKse
FGHsalIJKsp
FGHsalIJKto
FGHsalIJKScentrooeste
FGHsalIJKSnordeste
FGHsalIJKSnorte}


system("cargo make build_release")

sedes.each_with_index do |sede,i|

    port = 3029+i

    url = "https://global.naquadah.com.br/bocasecret/admin/report/webcast.php?webcastcode=#{sede}"

    command = "cargo run --release -p lib-server #{port} #{url}"

    system("echo #{command}")
    if i==0
        system("tmux new -d #{command}")
    else
        system("tmux neww #{command}")
    end
end