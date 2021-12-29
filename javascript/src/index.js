import { Howl, Howler } from "howler";

export function play(file) {
  var sound = new Howl({
    src: [file],
  });
  sound.play();
}

export function stop() {
  Howler.stop();
}
