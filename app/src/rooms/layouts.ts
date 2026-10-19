import type { Section } from "../domain/paper";
export const layouts: Record<
  Section,
  {
    wall: number;
    accent: number;
    floor: number;
    label: string;
    subtitle: string;
  }
> = {
  listening: {
    wall: 0xd4dcd5,
    accent: 0x345650,
    floor: 0x8e826e,
    label: "Listening room",
    subtitle: "安静听音室 · 耳机与独立书桌",
  },
  reading: {
    wall: 0xe5dfd0,
    accent: 0x6b7753,
    floor: 0x9b8466,
    label: "Reading room",
    subtitle: "自然光阅读室 · 专注每一段文字",
  },
  writing: {
    wall: 0xd9dfe1,
    accent: 0x425d73,
    floor: 0x9d8974,
    label: "Writing room",
    subtitle: "独立写作室 · 留出思考的空间",
  },
  speaking: {
    wall: 0xd9dbd2,
    accent: 0x536858,
    floor: 0x928673,
    label: "Speaking room",
    subtitle: "角色运行时导入与云语音待接入",
  },
};
