import type { FC } from "react";

interface Tag {
  name: string;
  color: string;
}

interface Project {
  id: string;
  name: string;
  filename: string;
  timestamp: string;
  framework: string;
  tags: Tag[];
}

interface ProjectCardProps {
  project: Project;
}

const ProjectCard: FC<ProjectCardProps> = ({ project }) => {
  return (
    <div className="bg-[#1C1C1C] rounded-lg p-4 border border-white/10 transition-colors cursor-pointer hover:border-white/25">
      <div className="flex justify-between items-start mb-2">
        <h3 className="font-medium text-white">{project.name}</h3>
        <span className="text-xs text-white/25">{project.timestamp}</span>
      </div>
      <p className="text-sm text-white/50 mb-3">{project.filename}</p>
      <div className="flex gap-2">
        {project.tags.map((tag, index) => (
          <div
            key={index}
            className={`${tag.color} text-white text-xs px-2 py-1 rounded flex items-center`}
          >
            {tag.name === "solana" && <span className="mr-1">◎</span>}
            {tag.name}
          </div>
        ))}
      </div>
    </div>
  );
};

export default ProjectCard;
