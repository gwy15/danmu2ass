<template>
  <v-app>
    <v-app-bar app color="primary" dark>
      <div class="d-flex align-center">Danmu2ass</div>

      <v-spacer></v-spacer>

      <v-btn href="https://github.com/gwy15/danmu2ass" target="_blank" text>
        <span class="mr-2">GitHub</span>
        <v-icon>mdi-github</v-icon>
      </v-btn>
    </v-app-bar>

    <v-main>
      <HomeView />
    </v-main>
  </v-app>
</template>

<script lang="ts">
import Vue from "vue";
import HomeView from "./components/Home.vue";

export default Vue.extend({
  name: "App",

  components: {
    HomeView,
  },

  data: () => ({
    //
  }),
});
</script>
