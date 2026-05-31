import { parseCgiDataNewOnServer } from '#shared/utils/html';
import { samples, read } from './common';

async function run() {
  for (const group of samples.filter(group => group.hasContent)) {
    console.group(group.name);
    for (const samplePath of group.samples) {
      const html = read(samplePath);
      const cgiData = await parseCgiDataNewOnServer(html);
      if (!cgiData) {
        console.warn('提取 window.cgiDataNew 对象失败');
        continue;
      }
      console.log(cgiData.item_show_type);
    }
    console.groupEnd();
    console.log();
  }
}

run();
