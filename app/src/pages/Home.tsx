import {
  ArrowUpRight,
  Headphones,
  BookOpen,
  PenLine,
  Mic,
  Layers,
  GraduationCap,
} from "lucide-react";
export const sectionLabels = {
  listening: "听力",
  reading: "阅读",
  writing: "写作",
  speaking: "口语",
} as const;
export function Home({
  onMode,
}: {
  onMode: (mode: "exam" | "training") => void;
}) {
  const icons = [Headphones, BookOpen, PenLine, Mic];
  return (
    <>
      <div className="eyebrow">
        YOUR PERSONAL EXAM SPACE <span className="pill">Academic</span>
      </div>
      <section className="hero">
        <div>
          <p className="overline">从容练习，走进考场。</p>
          <h1>
            让每一次练习，
            <br />
            更接近你的目标。
          </h1>
          <p className="muted">
            在独立考场里专注学习，
            <br />
            用完整的记录，看见每一步进展。
          </p>
          <div className="hero-actions">
            <button className="primary" onClick={() => onMode("training")}>
              训练模式 <ArrowUpRight size={17} />
            </button>
            <button onClick={() => onMode("exam")}>
              考试模式 <ArrowUpRight size={17} />
            </button>
          </div>
        </div>
        <div className="hero-art" aria-hidden="true">
          <div className="art-window" />
          <div className="art-desk" />
          <div className="art-paper">
            <span>IELTS</span>
            <b>Academic</b>
            <i />
            <i />
            <i />
            <i />
          </div>
          <div className="art-pen" />
          <span className="art-caption">A SPACE TO FOCUS</span>
        </div>
      </section>
      <div className="section-heading">
        <h2>四项能力，一个学习空间</h2>
        <span>LISTEN · READ · WRITE · SPEAK</span>
      </div>
      <div className="section-grid">
        {Object.entries(sectionLabels).map(([id, label], i) => {
          const Icon = icons[i];
          return (
            <article className="section-card" key={id}>
              <span className="number">0{i + 1}</span>
              <Icon size={25} />
              <h3>{label}</h3>
              <p>
                {
                  [
                    "听见细节，跟上语速",
                    "读懂结构，定位依据",
                    "组织思路，准确表达",
                    "面对考官，自然交流",
                  ][i]
                }
              </p>
              <small>{id.toUpperCase()}</small>
            </article>
          );
        })}
      </div>
      <div className="notice">
        <Layers size={20} />
        <div>
          <b>本地学习空间</b>
          <p>资产与学习记录可分别保存。开发验证卷仅用于体验流程。</p>
        </div>
        <GraduationCap size={24} />
      </div>
    </>
  );
}
