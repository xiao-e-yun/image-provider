import images from "./images.json";

while (images.length > 0) {
  const responseList: Promise<Response>[] = [];
  for (let index = 0; index < 10; index++) {
    const image = images.shift();
    if (!image) throw new Error("No image");
    responseList.push(fetch(image.href + "?w=500&h=300&output=webp"));
  }
  const time = Date.now();
  await Promise.allSettled(responseList).then(() => {
    console.log("time", Date.now() - time);
  });
}
