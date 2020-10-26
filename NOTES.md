

Para rodar camera virtual no linux, usando o obs, são necessárias 2 dependências:

```
community/v4l2loopback-dkms
aur/obs-v4l2sink
```

Para poder compartilhar a cam virtual no Chrome, é necessário carregar o módul usando a opção exclusive_caps=1

```
sudo modprobe v4l2loopback exclusive_caps=1
```

