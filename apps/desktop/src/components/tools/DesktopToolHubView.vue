<script setup lang="ts">
  import DesktopUiIcon from "@/components/ui/DesktopUiIcon.vue";

  type DesktopToolCard = {
    id: "video" | "audio" | "knowledge";
    title: string;
    description: string;
    status: string;
    icon: "video" | "audio" | "tools";
    disabled?: boolean;
  };

  const emit = defineEmits<{
    openTool: [toolId: DesktopToolCard["id"]];
  }>();

  const toolCards: DesktopToolCard[] = [
    {
      id: "video",
      title: "视频播放器",
      description:
        "添加本地视频目录，按树结构浏览视频，记住上次播放的视频和位置。",
      status: "可用",
      icon: "video",
    },
    // {
    //   id: "audio",
    //   title: "音频工具",
    //   description: "未来用于管理本地音频、课程录音和播客资料。",
    //   status: "规划中",
    //   icon: "audio",
    //   disabled: true,
    // },
    // {
    //   id: "knowledge",
    //   title: "知识库问答",
    //   description: "后续接入 LLM，对本地文档和学习资料进行问答。",
    //   status: "下一阶段",
    //   icon: "tools",
    //   disabled: true,
    // },
  ];
</script>

<template>
  <section class="desktop-tool-hub">
    <header class="desktop-tool-hub__header">
      <p class="desktop-tool-hub__kicker">Tools</p>
      <h2 class="desktop-tool-hub__title">工具中心</h2>
      <p class="desktop-tool-hub__summary">
        工具模块独立于阅读模块，只展示工具的入口。
      </p>
    </header>

    <div class="desktop-tool-hub__grid">
      <button
        v-for="card in toolCards"
        :key="card.id"
        :class="[
          'desktop-tool-hub__card',
          { 'desktop-tool-hub__card--disabled': card.disabled },
        ]"
        type="button"
        :disabled="card.disabled"
        @click="emit('openTool', card.id)"
      >
        <span class="desktop-tool-hub__card-top">
          <span class="desktop-tool-hub__icon">
            <DesktopUiIcon :name="card.icon" :size="24" />
          </span>
          <span class="desktop-tool-hub__badge">{{ card.status }}</span>
        </span>

        <span class="desktop-tool-hub__copy">
          <strong>{{ card.title }}</strong>
          <span>{{ card.description }}</span>
        </span>

        <span v-if="!card.disabled" class="desktop-tool-hub__action">
          打开工具
          <DesktopUiIcon name="chevron-right" :size="14" />
        </span>
      </button>
    </div>
  </section>
</template>

<style scoped>
  .desktop-tool-hub {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    gap: 1.25rem;
    min-width: 0;
    min-height: 0;
    height: 100%;
    padding: 2rem 2.2rem;
    background:
      radial-gradient(
        circle at top left,
        rgba(var(--desktop-accent-rgb), 0.12),
        transparent 25%
      ),
      linear-gradient(
        180deg,
        rgba(var(--desktop-accent-rgb), 0.035),
        transparent 20%
      ),
      var(--desktop-bg);
  }

  .desktop-tool-hub__header {
    display: grid;
    gap: 0.32rem;
    max-width: 44rem;
  }

  .desktop-tool-hub__kicker {
    margin: 0;
    color: var(--desktop-soft);
    font-size: 0.68rem;
    font-weight: 800;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .desktop-tool-hub__title {
    margin: 0;
    color: var(--desktop-ink);
    font-size: 1.55rem;
    font-weight: 760;
    letter-spacing: -0.035em;
  }

  .desktop-tool-hub__summary {
    margin: 0;
    color: var(--desktop-muted);
    font-size: 0.84rem;
    line-height: 1.65;
  }

  .desktop-tool-hub__grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 1rem;
    align-content: start;
    min-height: 0;
  }

  .desktop-tool-hub__card {
    display: grid;
    gap: 1.1rem;
    align-content: start;
    min-height: 13rem;
    padding: 1.2rem;
    border: 1px solid
      color-mix(in srgb, var(--desktop-line-strong) 72%, var(--desktop-line));
    border-radius: 22px;
    background:
      linear-gradient(
        135deg,
        rgba(var(--desktop-accent-rgb), 0.08),
        transparent 42%
      ),
      var(--desktop-surface-strong);
    color: var(--desktop-ink);
    text-align: left;
    box-shadow: 0 18px 46px rgba(var(--desktop-shadow), 0.08);
    cursor: pointer;
  }

  .desktop-tool-hub__card:hover:not(:disabled) {
    border-color: rgba(var(--desktop-accent-rgb), 0.28);
    background:
      linear-gradient(
        135deg,
        rgba(var(--desktop-accent-rgb), 0.12),
        transparent 46%
      ),
      var(--desktop-surface-strong);
  }

  .desktop-tool-hub__card:disabled {
    cursor: default;
  }

  .desktop-tool-hub__card--disabled {
    opacity: 0.58;
  }

  .desktop-tool-hub__card-top {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.8rem;
  }

  .desktop-tool-hub__icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 3.2rem;
    height: 3.2rem;
    border-radius: 18px;
    background: rgba(var(--desktop-accent-rgb), 0.09);
    color: var(--desktop-accent);
  }

  .desktop-tool-hub__badge {
    display: inline-flex;
    align-items: center;
    min-height: 1.55rem;
    padding: 0 0.56rem;
    border-radius: 999px;
    background: rgba(var(--desktop-accent-rgb), 0.07);
    color: var(--desktop-accent);
    font-size: 0.68rem;
    font-weight: 760;
  }

  .desktop-tool-hub__copy {
    display: grid;
    gap: 0.42rem;
  }

  .desktop-tool-hub__copy strong {
    font-size: 1rem;
    font-weight: 760;
  }

  .desktop-tool-hub__copy span {
    color: var(--desktop-muted);
    font-size: 0.8rem;
    line-height: 1.62;
  }

  .desktop-tool-hub__action {
    display: inline-flex;
    align-items: center;
    gap: 0.22rem;
    width: fit-content;
    color: var(--desktop-accent);
    font-size: 0.76rem;
    font-weight: 720;
  }

  @media (max-width: 1080px) {
    .desktop-tool-hub {
      padding: 1.25rem;
    }

    .desktop-tool-hub__grid {
      grid-template-columns: 1fr;
    }
  }
</style>
