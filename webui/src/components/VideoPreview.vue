<template>
  <div class="video-preview">
    <div v-if="!selectedVideo">
      <div
        class="dropzone"
        :class="{ dragover: showDropzoneMessage }"
        @dragover="handleDragOver"
        @dragleave="handleDragLeave"
        @drop="handleDrop"
      >
        <p>
          {{
            showDropzoneMessage ? "松手以选择预览视频" : "拖拽预览视频到此处"
          }}
        </p>
      </div>
    </div>
    <div v-else>
      <video ref="preview" width="100%" :src="selectedVideo" controls></video>
    </div>
  </div>
</template>
  
<script>
import SubtitlesOctopus from "libass-wasm";

export default {
  data() {
    return {
      selectedVideo: null,
      showDropzoneMessage: false,
      instance: null,
    };
  },
  props: {
    ass: {
      type: String,
      default: null,
    },
  },
  methods: {
    handleDragOver(event) {
      event.preventDefault();
      this.showDropzoneMessage = true;
    },
    handleDragLeave(event) {
      event.preventDefault();
      this.showDropzoneMessage = false;
    },
    handleDrop(event) {
      event.preventDefault();
      this.showDropzoneMessage = false;
      const file = event.dataTransfer.files[0];
      this.convertToSelectedVideo(file);
    },
    convertToSelectedVideo(file) {
      // 根据您的需求进行视频转换的逻辑
      // 这里只是简单地将文件路径设置为选定视频
      this.selectedVideo = URL.createObjectURL(file);
      this.$emit("video-selected", this.selectedVideo);
    },
  },
  watch: {
    ass(value) {
      if (!this.selectedVideo) {
        return;
      }
      if (!value) {
        return;
      }
      if (this.instance) {
        console.debug("Updating sub");
        this.instance.setTrack(value);
        return;
      }
      let video = this.$refs["preview"];
      let options = {
        video,
        subContent: value,
        workerUrl: "/js/subtitles-octopus-worker.js",
        availableFonts: {
          黑体: "/fonts/simhei.ttf",
        },
        fallbackFont: "/fonts/fallback.ttf",
      };
      console.log("set options", options);
      this.instance = new SubtitlesOctopus(options);
    },
  },
};
</script>
  
<style lang="less" scoped>
.video-preview {
  width: 100%;
  .dropzone {
    border: 2px dashed rgba(128, 128, 128, 0.6);
    border-radius: 8px;
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 400px;
  }

  .dragover {
    background-color: #f0f0f0;
  }

  .dropzone p {
    margin: 0;
  }
}
</style>