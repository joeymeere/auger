"use client";

import { useState } from "react";
import Layout from "@/components/layout";
import ProjectCard from "@/components/project-card";
import { Cog } from "lucide-react";

export default function ProjectsPage() {
  const [activeTab, setActiveTab] = useState("recent");

  const [tabs, setTabs] = useState([
    { id: "tab1", label: "my_awesome_project", active: true },
    { id: "tab2", label: "another_project", active: false },
  ]);

  const handleTabChange = (tabId: string) => {
    setTabs(
      tabs.map((tab) => ({
        ...tab,
        active: tab.id === tabId,
      }))
    );
  };

  const handleTabClose = (tabId: string) => {
    const newTabs = tabs.filter((tab) => tab.id !== tabId);
    // If we closed the active tab, activate the first remaining tab
    if (tabs.find((tab) => tab.id === tabId)?.active && newTabs.length > 0) {
      newTabs[0].active = true;
    }
    setTabs(newTabs);
  };

  const projects = [
    {
      id: "my_awesome_project",
      name: "my_awesome_project",
      filename: "program.so",
      timestamp: "Just now",
      framework: "anchor",
      tags: [
        { name: "solana", color: "bg-[#FF32C6]/50 border border-[#FF32C6]/75" },
        { name: "anchor", color: "bg-[#004C91]" },
      ],
    },
    {
      id: "another_project",
      name: "another_project",
      filename: "native_program.so",
      timestamp: "Just now",
      framework: "native",
      tags: [
        { name: "solana", color: "bg-[#FF32C6]/50 border border-[#FF32C6]/75" },
        { name: "native", color: "bg-white !text-black" },
      ],
    },
    {
      id: "real_boy",
      name: "real_boy",
      filename: "pinn_program.so",
      timestamp: "Just now",
      framework: "pinocchio",
      tags: [
        { name: "solana", color: "bg-[#FF32C6]/50 border border-[#FF32C6]/75" },
        { name: "pinocchio", color: "bg-[#27272A] border border-white/10" },
      ],
    },
    // Add more projects with different frameworks for demonstration
    {
      id: "another_anchor_project",
      name: "another_anchor_project",
      filename: "program.so",
      timestamp: "Just now",
      framework: "anchor",
      tags: [
        { name: "solana", color: "bg-[#FF32C6]/50 border border-[#FF32C6]/75" },
        { name: "anchor", color: "bg-[#004C91]" },
      ],
    },
    {
      id: "another_native_project",
      name: "another_native_project",
      filename: "native_program.so",
      timestamp: "Just now",
      framework: "native",
      tags: [
        { name: "solana", color: "bg-[#FF32C6]/50 border border-[#FF32C6]/75" },
        { name: "native", color: "bg-white !text-black" },
      ],
    },
    {
      id: "another_pinocchio_project",
      name: "another_pinocchio_project",
      filename: "pinn_program.so",
      timestamp: "Just now",
      framework: "pinocchio",
      tags: [
        { name: "solana", color: "bg-[#FF32C6]/50 border border-[#FF32C6]/75" },
        { name: "pinocchio", color: "bg-[#27272A] border border-white/10" },
      ],
    },
  ];

  // Group projects by framework
  const getProjectsByFramework = () => {
    const grouped: Record<string, typeof projects> = {};

    projects.forEach((project) => {
      const framework = project.framework;
      if (!grouped[framework]) {
        grouped[framework] = [];
      }
      grouped[framework].push(project);
    });

    // Convert to array of framework groups with capitalized names
    return Object.entries(grouped).map(([framework, projects]) => ({
      name: framework.charAt(0).toUpperCase() + framework.slice(1),
      projects,
    }));
  };

  return (
    <Layout
      tabs={tabs}
      title={""}
      onTabChange={handleTabChange}
      onTabClose={handleTabClose}
    >
      <div className="flex min-h-[93vh]">
        <div className="flex-1 flex-col p-8">
          <div className="flex flex-1 items-center justify-between pb-2">
            <div className="mb-2">
              <h1 className="text-3xl">My Projects</h1>
              <p className="text-white/50 font-light mt-1">
                View and manage the projects you've created
              </p>
            </div>

            <div className="">
              <div className="inline-flex rounded-lg bg-[#1c1c1c] p-1">
                <button
                  className={`px-4 py-2 text-sm rounded-md ${
                    activeTab === "recent"
                      ? "bg-[#0D0D0D] text-white"
                      : "text-white/40"
                  }`}
                  onClick={() => setActiveTab("recent")}
                >
                  Recent
                </button>
                <button
                  className={`px-4 py-2 text-sm rounded-md ${
                    activeTab === "byType"
                      ? "bg-[#0D0D0D] text-white"
                      : "text-white/40"
                  }`}
                  onClick={() => setActiveTab("byType")}
                >
                  By Type
                </button>
              </div>
            </div>
          </div>
          <div className="w-20 border-t border-[#27272A]" />

          {activeTab === "recent" ? (
            <div className="mt-8 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
              {projects.map((project) => (
                <ProjectCard key={project.id} project={project} />
              ))}
            </div>
          ) : (
            <div className="mt-8 space-y-12">
              {getProjectsByFramework().map((group) => (
                <div key={group.name} className="">
                  <h2 className="text-xl">{group.name}</h2>
                  <div className="w-10 mt-2 mb-4 border-t border-[#27272A]" />
                  <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                    {group.projects.map((project) => (
                      <ProjectCard key={project.id} project={project} />
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="w-80 min-h-full border-l border-[#27272a]">
          <div className="border-b border-[#27272a] p-4">
            <h2 className="text-xl font-semibold">Pinned</h2>
          </div>
          <div className="p-4 mt-4">
            <div className="flex flex-col items-center justify-center h-80 text-center">
              <div className="p-2 rounded-lg bg-gradient-to-b from-[#1C1C1C] to-[#111111] border border-white/10 mb-4">
                <Cog className="w-6 h-6 text-white/10" />
              </div>
              <h3 className="text-xl font-semibold mb-2">No Pinned Projects</h3>
              <p className="max-w-sm text-white/30 text-xs">
                Right-click on a project, and click "Pin project" to populate
                this section.
              </p>
            </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}
