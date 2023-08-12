<template>
  <v-container class="danmu2ass">
    <v-text-field
      class="source"
      dense
      hide-details
      outlined
      v-model="display_source"
      label="选择 xml 或输入 URL / BV 号 / ep 号"
      append-icon="mdi-file-upload-outline"
      @click:append="select_xml"
    />

    <config-editor class="config" v-model="config">
      <v-col cols="6">
        <v-btn
          id="render-preview"
          :disabled="!video_loaded || source === null"
          @click="render_ass"
          :loading="loading"
        >
          <v-icon>mdi-eye-outline</v-icon>
          渲染预览
        </v-btn>
        <v-btn
          id="download-ass"
          :disabled="source === null"
          @click="download_ass"
          :loading="loading"
        >
          <v-icon>mdi-download</v-icon>
          下载 ASS 文件
        </v-btn>
      </v-col>
    </config-editor>
    <div class="preview">
      <video-preview
        :config="config"
        @video-selected="video_loaded = true"
        :ass="preview_ass"
      ></video-preview>
    </div>
  </v-container>
</template>

<script lang="ts">
import Vue from "vue";
import ConfigEditor from "./ConfigEditor.vue";
import { IConfig, load_config, save_config } from "@/config";
import VideoPreview from "./VideoPreview.vue";

function selectXmlFile(): Promise<[string, string]> {
  return new Promise<[string, string]>((resolve, reject) => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".xml";

    input.onchange = (event: any) => {
      const file = event.target.files[0];
      const name = file.name;
      const reader = new FileReader();

      reader.onload = () => {
        const contents = reader.result as string;
        resolve([contents, name]);
      };

      reader.onerror = (error) => {
        reject(error);
      };

      reader.readAsText(file);
    };

    input.click();
  });
}

interface IData {
  source_type: "xml" | "url" | null;
  source: string | null;
  xml_title: string | null;
  config: IConfig;
  video_loaded: boolean;
  loading: boolean;
  preview_ass: string | null;
}

export default Vue.extend({
  name: "HomeView",

  components: {
    ConfigEditor,
    VideoPreview,
  },

  data: () =>
    ({
      source_type: null,
      source: null,
      xml_title: null,
      config: load_config(),
      video_loaded: false,
      loading: false,
      preview_ass: null,
    } as IData),

  methods: {
    select_xml() {
      selectXmlFile().then(([contents, name]) => {
        console.log("selected an xml file");
        this.source_type = "xml";
        this.source = contents;
        this.xml_title = name;
      });
    },
    async get_response() {
      this.loading = true;
      let source = {
        type: this.source_type,
      } as any;

      if (this.source_type === "xml") {
        source.content = {
          content: this.source,
          title: this.xml_title,
        };
      } else if (this.source_type === "url") {
        source.content = { url: this.source };
      }
      let url =
        process.env.NODE_ENV === "production"
          ? "/convert"
          : "http://127.0.0.1:8081/convert";
      let response = await fetch(url, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          source,
          config: this.config,
        }),
      });
      return response;
    },
    async download_ass() {
      try {
        let response = await this.get_response();
        // down response as {title}.ass
        let blob = await response.blob();
        const contentDisposition = response.headers.get("content-disposition")!;
        console.debug("header", contentDisposition);
        const fileNameMatch = contentDisposition.match(/filename="(.+)"/);
        const fileName = fileNameMatch ? fileNameMatch[1] : "字幕.ass";
        console.log(fileName);
        // 创建临时下载链接
        const url = URL.createObjectURL(blob);
        // 创建一个<a>标签
        const link = document.createElement("a");
        link.href = url;
        link.download = fileName; // 设置下载的文件名
        // 模拟点击链接以触发下载
        link.click();
        // 清理临时下载链接
        URL.revokeObjectURL(url);
      } finally {
        this.loading = false;
      }
    },
    async render_ass() {
      this.preview_ass = null;
      try {
        let response = await this.get_response();
        let content = await response.text();
        this.preview_ass = content;
      } finally {
        this.loading = false;
      }
    },
  },
  watch: {
    config: {
      handler(val) {
        save_config(val);
      },
      deep: true,
    },
  },
  computed: {
    display_source: {
      get() {
        if (this.source_type === "xml") {
          const encoder = new TextEncoder();
          const encodedData = encoder.encode(this.source!);
          const utf8Length = encodedData.length;
          return `已选择 ${(utf8Length / 1024 / 1024).toFixed(
            2
          )} MiB 的 xml 文件`;
        } else if (this.source_type === "url") {
          return this.source;
        } else {
          return "";
        }
      },
      set(newValue: string) {
        // check if it's a url
        this.source_type = "url";
        this.source = newValue;
      },
    },
  },
});
</script>

<style lang="less" scoped>
.source {
  margin-bottom: 20px;
}
.config {
  margin-bottom: 20px;
}
#render-preview {
  margin-right: 10px;
}
.preview {
  width: 100%;
  min-height: 400px;
}
</style>

<style>
.container.danmu2ass {
  max-width: 1000px;
}
</style>
