import type { QuestionKind } from "../domain/paper";
export const questionLabels: Record<QuestionKind, string> = {
  single_choice: "单项选择",
  multiple_choice: "多项选择",
  matching: "匹配",
  gap_fill: "填空",
  short_answer: "简答",
  diagram_label: "图表标注",
  true_false_not_given: "判断事实",
  yes_no_not_given: "判断观点",
  heading_match: "标题匹配",
  writing_task: "写作",
  speaking_prompt: "口语提示",
};
